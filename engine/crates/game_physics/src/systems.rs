use crate::{
    bodies::AxisAlignedBoundingBox,
    broad_phase::{BroadPhase, Entry},
    Acceleration, BodyType, BroadPhaseQuadTree, Drag, EntityCollision, Forces, Gravity, Mass,
    PhysicsState, TileCollision, Velocity,
};
use bevy::{
    ecs::ShouldRun,
    prelude::*,
    tasks::{ComputeTaskPool, ParallelIterator},
};
use game_tiles::{GetTileError, Tile, TilePosition, TileWorld};
use tracing::instrument;

#[instrument(skip(commands))]
pub fn setup(commands: &mut Commands) {
    commands.insert_resource(BroadPhaseQuadTree::new(AxisAlignedBoundingBox::new(
        Vec2::zero(),
        // TODO: change the size to something more accurate
        Vec2::new(256.0, 256.0),
    )));
}

#[instrument(skip(time, state))]
pub fn update_state(time: Res<Time>, mut state: ResMut<PhysicsState>) {
    state.lag += time.delta();

    if state.lerp() >= 4.0 {
        warn!(
            "Physics simulation is behind by {} steps",
            state.lerp().floor()
        );
    }
}

#[instrument(skip(state, query))]
pub fn add_kinematic_forces(
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
pub fn apply_forces(mut query: Query<(&mut Forces, &mut Acceleration, &Mass)>) {
    for (mut forces, mut acceleration, mass) in query.iter_mut() {
        for force in forces.0.drain(..) {
            // a = F/m
            acceleration.0 += force / mass.0;

            // Verify that the acceleration is valid
            if !acceleration.0.is_finite() {
                error!("acceleration is not finite: {:?}", acceleration);
                acceleration.0 = Vec2::zero();
            }
        }
    }
}

#[instrument(skip(time, query))]
pub fn apply_acceleration(
    time: Res<Time>,
    state: Res<PhysicsState>,
    mut query: Query<(&Acceleration, &mut Velocity)>,
) {
    for (acceleration, mut velocity) in query.iter_mut() {
        // v = a * t
        velocity.0 += acceleration.0 * time.delta_seconds();
    }
}

#[instrument(skip(tile_collisions, entity_collisions))]
pub fn reset_events(
    mut tile_collisions: ResMut<Events<TileCollision>>,
    mut entity_collisions: ResMut<Events<EntityCollision>>,
) {
    tile_collisions.update();
    entity_collisions.update();
}

#[instrument(skip(state))]
pub fn should_step(state: Res<PhysicsState>) -> ShouldRun {
    if state.lag >= state.interval {
        ShouldRun::YesAndLoop
    } else {
        ShouldRun::No
    }
}

#[instrument(skip(pool, quadtree, state, entity_collisions, world, query))]
pub fn step(
    pool: Res<ComputeTaskPool>,
    quadtree: Res<BroadPhaseQuadTree>,
    mut state: ResMut<PhysicsState>,
    mut tile_collisions: ResMut<Events<TileCollision>>,
    mut entity_collisions: ResMut<Events<EntityCollision>>,
    world: Res<TileWorld>,
    mut query: Query<(
        Entity,
        &mut AxisAlignedBoundingBox,
        &mut Velocity,
        &BodyType,
    )>,
) {
    let state = &mut *state;
    state.lag -= state.interval;

    #[derive(Clone, Debug)]
    enum Collision {
        Entity(EntityCollision),
        Tile(TileCollision),
    }

    let (tx, rx) = crossbeam::channel::unbounded();
    let quadtree = &*quadtree;
    let state = &*state;
    let world = &*world;
    info_span!("collisions").in_scope(|| {
        query.par_iter_mut(25).for_each(
            &pool,
            move |(entity, mut bounds, mut velocity, &body_type)| {
                // Only step on kinematic bodies
                if body_type != BodyType::Kinematic {
                    return;
                }

                #[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
                enum CollisionAxis {
                    X,
                    Y,
                }

                #[derive(Clone, PartialEq, Debug)]
                struct CollisionData {
                    axis: CollisionAxis,
                    tile: Tile,
                    tile_position: TilePosition,
                }

                // Do tile collisions and calculate next bounds
                let (next_bounds, collision_data) = info_span!("tiles").in_scope(|| {
                    // Calculate next bounds
                    let next_bounds: AxisAlignedBoundingBox =
                        *bounds + velocity.0 * state.interval.as_secs_f32();

                    // Get intersecting tiles
                    let intersections = world
                        .iter_intersecting(next_bounds.bottom_left(), next_bounds.top_right())
                        .flat_map(|(position, result)| {
                            match result {
                                Ok(tile) => tile,
                                Err(GetTileError::OutOfBounds(..)) => None,
                                Err(error) => {
                                    error!(
                                        ?error,
                                        "failure getting tile for collision checking: {}", error
                                    );
                                    None
                                }
                            }
                            .map(move |tile| (position, tile))
                        });

                    // Calculate intersection depths
                    let intersection_times = intersections
                        .flat_map(|(position, tile)| {
                            let tile_bounds =
                                AxisAlignedBoundingBox::new(position.into(), Vec2::one());

                            // Time (in seconds) to collide along x-axis
                            let x_time = if velocity.0[0] > 0.0 {
                                Some(tile_bounds.left() - bounds.right()).filter(|&d| d >= 0.0)
                            } else if velocity.0[0] < 0.0 {
                                Some(tile_bounds.right() - bounds.left()).filter(|&d| d <= 0.0)
                            } else {
                                None
                            }
                            .map(|d| {
                                (
                                    d / velocity.0[0],
                                    CollisionData {
                                        axis: CollisionAxis::X,
                                        tile,
                                        tile_position: position,
                                    },
                                )
                            });

                            // Time (in seconds) to collide along y-axis
                            let y_time = if velocity.0[1] > 0.0 {
                                Some(tile_bounds.bottom() - bounds.top()).filter(|&d| d >= 0.0)
                            } else if velocity.0[1] < 0.0 {
                                Some(tile_bounds.top() - bounds.bottom()).filter(|&d| d <= 0.0)
                            } else {
                                None
                            }
                            .map(|d| {
                                (
                                    d / velocity.0[1],
                                    CollisionData {
                                        axis: CollisionAxis::Y,
                                        tile,
                                        tile_position: position,
                                    },
                                )
                            });

                            std::iter::once(x_time)
                                .chain(std::iter::once(y_time))
                                .flatten()
                        })
                        .filter(|(t, ..)| t.is_finite());

                    // Calculate minimum intersection time (first tile collision)
                    let min_time =
                        intersection_times.min_by(|(a, ..), (b, ..)| a.partial_cmp(b).unwrap());
                    min_time
                        .map(|(t, collision_data)| (*bounds + velocity.0 * t, Some(collision_data)))
                        .unwrap_or((next_bounds, None))
                });

                // Resolve tile collision
                if let Some(collision_data) = collision_data {
                    // Send collision event
                    tx.send(Collision::Tile(TileCollision {
                        entity,
                        entity_velocity: velocity.0,
                        tile: collision_data.tile,
                        tile_position: collision_data.tile_position,
                    }))
                    .unwrap();

                    // Add friction while standing on a surface
                    const FRICTION_COEFFICIENT: f32 = 0.8;
                    if collision_data.axis == CollisionAxis::Y && velocity.0[1] < 0.0 {
                        // simplified friction, normal friction is F = mu * N
                        velocity.0[0] *= FRICTION_COEFFICIENT;
                    }

                    // Adjust velocity due to tile collision
                    match collision_data.axis {
                        CollisionAxis::X => velocity.0[0] = 0.0,
                        CollisionAxis::Y => velocity.0[1] = 0.0,
                    }
                }

                // Update entity position
                *bounds = next_bounds;

                let bounds = *bounds;
                info_span!("entities").in_scope(|| {
                    // Broad phase
                    let broad_collisions = quadtree.query_bounds(bounds);

                    // Narrow phase
                    let narrow_collisions = broad_collisions
                        .filter(|&entry| {
                            quadtree
                                .get_bounds(entry)
                                .filter(|other| bounds.intersects(other))
                                .is_some()
                        })
                        .filter_map(|entry| quadtree.get(entry).copied());

                    for other_entity in narrow_collisions {
                        tx.send(Collision::Entity(EntityCollision {
                            entities: (entity, other_entity),
                        }))
                        .unwrap();
                    }
                });
            },
        );
    });

    info_span!("process_collisions").in_scope(|| {
        for event in rx {
            match event {
                Collision::Tile(event) => tile_collisions.send(event),
                Collision::Entity(event) => entity_collisions.send(event),
            }
        }
    });
}

#[instrument(skip(query))]
pub fn reset(mut query: Query<&mut Acceleration>) {
    for mut acceleration in query.iter_mut() {
        acceleration.0 = Vec2::zero();
    }
}

#[instrument(skip(query))]
pub fn update_transforms(
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
pub fn update_bodies(
    mut quadtree: ResMut<BroadPhaseQuadTree>,
    query: Query<(&Entry, &AxisAlignedBoundingBox), Mutated<AxisAlignedBoundingBox>>,
) {
    for (&entry, &bounds) in query.iter() {
        quadtree.set_bounds(entry, bounds);
    }
}

#[instrument(skip(commands, state, quadtree, query))]
pub fn add_bodies(
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
pub fn remove_bodies(
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
