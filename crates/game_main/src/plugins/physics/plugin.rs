use crate::plugins::physics::{
    Acceleration, AxisAlignedBoundingBox, BodyType, CollisionEvent, Drag, Entry, Forces, Gravity,
    Mass, PhysicsState, QuadTree, Velocity,
};
use bevy::{
    app::startup_stage,
    ecs::ShouldRun,
    prelude::*,
    tasks::{ComputeTaskPool, ParallelIterator},
    utils::HashMap,
};
use tracing::instrument;

pub type BroadPhaseQuadTree = QuadTree<Entity, 1, 4, 10>;

pub struct PhysicsPlugin;

impl PhysicsPlugin {
    /// Stage that runs before any physics steps are taken.
    pub const PRE_PHYSICS_STAGE: &'static str = "pre_physics";

    /// Stage that runs every physics step (0..N times per tick).
    pub const PHYSICS_STAGE: &'static str = "physics";

    /// Stage that runs after every physics step.
    pub const POST_PHYSICS_STAGE: &'static str = "post_physics";

    #[instrument(skip(commands))]
    fn setup(commands: &mut Commands) {
        commands.insert_resource(BroadPhaseQuadTree::new(AxisAlignedBoundingBox::new(
            Vec2::zero(),
            // TODO: change the size to something more accurate
            Vec2::new(256.0, 256.0),
        )));
    }

    #[instrument(skip(time, state))]
    fn update_state(time: Res<Time>, mut state: ResMut<PhysicsState>) {
        state.lag += time.delta();

        if state.lerp() >= 4.0 {
            warn!(
                "Physics simulation is behind by {} steps",
                state.lerp().floor()
            );
        }
    }

    #[instrument(skip(state, query))]
    fn add_kinematic_forces(
        state: Res<PhysicsState>,
        mut query: Query<(
            &mut Forces,
            &Mass,
            &Velocity,
            Option<&Gravity>,
            Option<&Drag>,
        )>,
    ) {
        for (mut forces, mass, velocity, gravity, drag) in query.iter_mut() {
            // Weight
            // W = m * g
            let gravity = gravity.map(|x| x.0).unwrap_or(state.gravity.0);
            forces.0.push(gravity * mass.0);

            // Simplified drag
            // D = (1/2) * C_d * r * A * V^2 = b * v^2
            let drag = drag.map(|x| x.0).unwrap_or(state.drag.0);
            let square_velocity = velocity.0 * velocity.0 * -velocity.0.signum();
            forces.0.push(drag * square_velocity);
        }
    }

    #[instrument(skip(query))]
    fn apply_forces(mut query: Query<(&mut Forces, &mut Acceleration, &Mass)>) {
        for (mut forces, mut acceleration, mass) in query.iter_mut() {
            for force in forces.0.drain(..) {
                // a = F/m
                acceleration.0 += force / mass.0;

                // Verify that the acceleration is valid
                if !acceleration.0.is_finite() {
                    error!("acceleration is not finite: {:?}", acceleration);
                    panic!("invalid acceleration value");
                }
            }
        }
    }

    #[instrument(skip(time, query))]
    fn apply_acceleration(
        time: Res<Time>,
        state: Res<PhysicsState>,
        mut query: Query<(&Acceleration, &mut Velocity)>,
    ) {
        for (acceleration, mut velocity) in query.iter_mut() {
            // v = a * t
            velocity.0 += acceleration.0 * time.delta_seconds();
        }
    }

    #[instrument(skip(state))]
    fn should_step(state: Res<PhysicsState>) -> ShouldRun {
        if state.lag >= state.interval {
            ShouldRun::YesAndLoop
        } else {
            ShouldRun::No
        }
    }

    #[instrument(skip(pool, quadtree, state, collisions, queries))]
    fn step(
        pool: Res<ComputeTaskPool>,
        quadtree: Res<BroadPhaseQuadTree>,
        mut state: ResMut<PhysicsState>,
        mut collisions: ResMut<Events<CollisionEvent>>,
        mut queries: QuerySet<(
            Query<(Entity, &AxisAlignedBoundingBox, &Velocity, &BodyType)>,
            Query<(
                Entity,
                &mut AxisAlignedBoundingBox,
                &mut Velocity,
                &BodyType,
            )>,
        )>,
    ) {
        let state = &mut *state;
        state.lag -= state.interval;
        collisions.update();

        // Broad phase
        let quadtree = &*quadtree;
        let (tx, rx) = crossbeam::channel::unbounded();
        let q0: &Query<_> = queries.q0();
        q0.par_iter(32)
            .for_each(&pool, move |(entity, &bounds, velocity, &body_type)| {
                if body_type != BodyType::Kinematic {
                    return;
                }

                let target_bounds = bounds + velocity.0;
                let search_bounds =
                    AxisAlignedBoundingBox::from_children(&[bounds, target_bounds]).unwrap();
                let results: Vec<_> = quadtree
                    .query_bounds(search_bounds)
                    .filter_map(|entry| quadtree.get(entry))
                    .collect();
                tx.send((entity, results)).unwrap();
            });

        // Collect all broad phase collisions
        let broad_collisions: HashMap<_, _> = rx.into_iter().collect();

        let q1: &mut Query<_> = queries.q1_mut();
        let state = &*state;
        q1.par_iter_mut(32).for_each(
            &pool,
            move |(entity, mut bounds, mut velocity, body_type)| {
                *bounds += velocity.0 * state.interval.as_secs_f32();
            },
        );
    }

    #[instrument(skip(query))]
    fn reset(mut query: Query<&mut Acceleration>) {
        for mut acceleration in query.iter_mut() {
            acceleration.0 = Vec2::zero();
        }
    }

    #[instrument(skip(query))]
    fn update_transforms(
        state: Res<PhysicsState>,
        mut query: Query<(&mut Transform, &AxisAlignedBoundingBox, &Velocity)>,
    ) {
        for (mut transform, aabb, velocity) in query.iter_mut() {
            let center = aabb.center().extend(transform.translation[2]);
            let predicted_offset = velocity.0.extend(0.0) * state.lerp() * state.interval.as_secs_f32();
            transform.translation = center + predicted_offset;
            transform.translation;
        }
    }

    #[instrument(skip(quadtree, query))]
    fn update_bodies(
        mut quadtree: ResMut<BroadPhaseQuadTree>,
        query: Query<(&Entry, &AxisAlignedBoundingBox), Mutated<AxisAlignedBoundingBox>>,
    ) {
        for (&entry, &bounds) in query.iter() {
            quadtree.set_bounds(entry, bounds);
        }
    }

    #[instrument(skip(commands, state, quadtree, query))]
    fn add_bodies(
        commands: &mut Commands,
        mut state: ResMut<PhysicsState>,
        mut quadtree: ResMut<BroadPhaseQuadTree>,
        query: Query<(Entity, &AxisAlignedBoundingBox), Added<AxisAlignedBoundingBox>>,
    ) {
        for (entity, &bounds) in query.iter() {
            let entry = quadtree.insert(entity, bounds);
            commands.insert_one(entity, entry);
            state.entry_map.insert(entity, entry);
        }
    }

    #[instrument(skip(commands, state, quadtree, query))]
    fn remove_bodies(
        commands: &mut Commands,
        mut state: ResMut<PhysicsState>,
        mut quadtree: ResMut<BroadPhaseQuadTree>,
        query: Query<&Entry>,
    ) {
        let entities = query.removed::<AxisAlignedBoundingBox>();
        for &entity in entities {
            if let Some(&entry) = state.entry_map.get(&entity) {
                quadtree.remove(entry);
                commands.remove_one::<Entry>(entity);
                state.entry_map.remove(&entity);
            }
        }
    }
}

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<PhysicsState>()
            .add_event::<CollisionEvent>()
            .add_startup_system_to_stage(startup_stage::PRE_STARTUP, Self::setup.system())
            .add_stage_after(
                stage::UPDATE,
                Self::PRE_PHYSICS_STAGE,
                SystemStage::parallel()
                    .with_system(Self::update_bodies.system())
                    .with_system(Self::add_bodies.system())
                    .with_system(Self::update_state.system())
                    .with_system(Self::add_kinematic_forces.system())
                    .with_system(Self::apply_forces.system())
                    .with_system(Self::apply_acceleration.system()),
            )
            .add_stage_after(
                Self::PRE_PHYSICS_STAGE,
                Self::PHYSICS_STAGE,
                SystemStage::serial()
                    .with_run_criteria(Self::should_step.system())
                    .with_system(Self::step.system()),
            )
            .add_stage_after(
                Self::PHYSICS_STAGE,
                Self::POST_PHYSICS_STAGE,
                SystemStage::parallel()
                    .with_system(Self::reset.system())
                    .with_system(Self::update_transforms.system())
                    .with_system(Self::remove_bodies.system()),
            );
    }
}
