use crate::{
    Acceleration, BodyType, Drag, Forces, Gravity, JumpStatus, Mass, PhysicsState, TileCollision,
    TileCollisionAxis, Velocity,
};
use game_lib::{
    bevy::{ecs::schedule::ShouldRun, prelude::*, tasks::ComputeTaskPool},
    tracing::{self, instrument},
};
use game_tiles::{EntityWorldPosition, EntityWorldRect, GameWorld, TileWorldRect};

#[instrument(skip(commands, state))]
pub fn setup(mut commands: Commands, state: Option<Res<PhysicsState>>) {
    if state.is_none() {
        commands.insert_resource(PhysicsState::default());
    }
}

#[instrument(skip(commands))]
pub fn cleanup(mut commands: Commands) {
    commands.remove_resource::<PhysicsState>();
}

#[instrument(skip(time, state))]
pub fn update_physics_state(time: Res<Time>, mut state: ResMut<PhysicsState>) {
    state.queued_steps += state.step_timer.tick(time.delta()).times_finished();

    if state.queued_steps > 10 {
        warn!(
            "Physics simulation is behind by {} steps",
            state.queued_steps
        );
    }
}

#[instrument(skip(state))]
pub fn if_physics_lagged(state: Res<PhysicsState>) -> ShouldRun {
    if state.queued_steps > 0 {
        ShouldRun::Yes
    } else {
        ShouldRun::No
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
        forces.0.push(square_velocity * drag);
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
                acceleration.0 = EntityWorldPosition::ZERO;
            }
        }
    }
}

#[instrument(skip(time, query))]
pub fn apply_acceleration(time: Res<Time>, mut query: Query<(&Acceleration, &mut Velocity)>) {
    for (acceleration, mut velocity) in query.iter_mut() {
        // v = a * t
        velocity.0 += acceleration.0 * time.delta_seconds();
    }
}

#[instrument(skip(state))]
pub fn while_physics_lagged(state: Res<PhysicsState>) -> ShouldRun {
    if state.queued_steps > 0 {
        ShouldRun::YesAndCheckAgain
    } else {
        ShouldRun::No
    }
}

#[instrument(skip(pool, state, tile_collisions, world, bodies))]
pub fn step(
    pool: Res<ComputeTaskPool>,
    mut state: ResMut<PhysicsState>,
    mut tile_collisions: EventWriter<TileCollision>,
    world: Res<GameWorld>,
    mut bodies: Query<(Entity, &mut EntityWorldRect, &mut Velocity, &BodyType)>,
) {
    let state = &mut *state;

    // Decrement queued steps
    if let Some(queued_steps) = state.queued_steps.checked_sub(1) {
        state.queued_steps = queued_steps;
    }

    let (tile_collisions_tx, tile_collisions_rx) = game_lib::crossbeam::channel::unbounded();
    let state = &*state;
    let world = &*world;
    info_span!("tile_collisions").in_scope(|| {
        bodies.par_for_each_mut(
            &pool,
            25,
            |(entity, mut bounds, mut velocity, &body_type)| {
                // Only step on kinematic bodies
                if body_type != BodyType::Kinematic {
                    return;
                }

                // Calculate unobstructed movement amount
                let target_offset = velocity.0 * state.step_timer.duration().as_secs_f32();
                let mut next_bounds = *bounds;
                let mut next_velocity = velocity.0;
                const STEP: f32 = 1.0;

                // Get an iterator over the bounds that will be checked along x-axis
                let x_steps = std::iter::from_fn({
                    let mut remaining = target_offset.x.abs();
                    let offset_sign = target_offset.x.signum();
                    let mut cur_bounds = next_bounds;
                    move || {
                        if remaining <= 0.0 {
                            None
                        } else if remaining <= STEP {
                            cur_bounds =
                                cur_bounds.offset(EntityWorldPosition::X * remaining * offset_sign);
                            remaining = 0.0;
                            Some(cur_bounds)
                        } else {
                            cur_bounds =
                                cur_bounds.offset(EntityWorldPosition::X * STEP * offset_sign);
                            remaining -= STEP;
                            Some(cur_bounds)
                        }
                    }
                });

                for step in x_steps {
                    // Get rectangle of tiles to check
                    let mut checked_tiles = TileWorldRect::from(step);
                    if velocity.0.x > 0.0 {
                        checked_tiles.bottom_left.x += checked_tiles.size.x - 1;
                        checked_tiles.size.x = 1;
                    } else {
                        checked_tiles.size.x = 1;
                    };

                    // Get all collided tiles
                    let collisions = checked_tiles.iter_positions().flat_map(|position| {
                        world
                            .get_tile(position)
                            .into_iter()
                            .copied()
                            .flatten()
                            .map(move |tile| (tile, position))
                    });

                    // Resolve collisions
                    let (collided, step_bounds, step_velocity) = collisions.fold(
                        (false, step, next_velocity),
                        |(_, mut next_bounds, mut next_velocity),
                         (collision_tile, collision_pos)| {
                            // Send collision event
                            tile_collisions_tx
                                .send(TileCollision {
                                    entity,
                                    entity_velocity: velocity.0,
                                    axis: TileCollisionAxis::X,
                                    tile: collision_tile,
                                    tile_position: collision_pos,
                                })
                                .unwrap();

                            // Update next bounds
                            next_bounds.bottom_left.x = if velocity.0.x > 0.0 {
                                collision_pos.x as f32 - next_bounds.width()
                            } else {
                                collision_pos.x as f32 + 1.0
                            };

                            // Update next velocity
                            next_velocity.x = 0.0;

                            (true, next_bounds, next_velocity)
                        },
                    );

                    // Update position and velocity
                    next_bounds = step_bounds;
                    next_velocity = step_velocity;

                    // Don't handle anymore x-axis collisions
                    if collided {
                        break;
                    }
                }

                let y_steps = std::iter::from_fn({
                    let mut remaining = target_offset.y.abs();
                    let offset_sign = target_offset.y.signum();
                    let mut cur_bounds = next_bounds;
                    move || {
                        if remaining <= 0.0 {
                            None
                        } else if remaining <= STEP {
                            cur_bounds =
                                cur_bounds.offset(EntityWorldPosition::Y * remaining * offset_sign);
                            remaining = 0.0;
                            Some(cur_bounds)
                        } else {
                            cur_bounds =
                                cur_bounds.offset(EntityWorldPosition::Y * STEP * offset_sign);
                            remaining -= STEP;
                            Some(cur_bounds)
                        }
                    }
                });

                for step in y_steps {
                    // Get rectangle of tiles to check
                    let mut checked_tiles = TileWorldRect::from(step);
                    if velocity.0.y > 0.0 {
                        checked_tiles.bottom_left.y += checked_tiles.size.y - 1;
                        checked_tiles.size.y = 1;
                    } else {
                        checked_tiles.size.y = 1;
                    };

                    // Get all collided tiles
                    let collisions = checked_tiles.iter_positions().flat_map(|position| {
                        world
                            .get_tile(position)
                            .into_iter()
                            .copied()
                            .flatten()
                            .map(move |tile| (tile, position))
                    });

                    // Resolve collisions
                    let (collided, step_bounds, step_velocity) = collisions.fold(
                        (false, step, next_velocity),
                        |(_, mut next_bounds, mut next_velocity),
                         (collision_tile, collision_pos)| {
                            // Send collision event
                            tile_collisions_tx
                                .send(TileCollision {
                                    entity,
                                    entity_velocity: velocity.0,
                                    axis: TileCollisionAxis::Y,
                                    tile: collision_tile,
                                    tile_position: collision_pos,
                                })
                                .unwrap();

                            // Update next bounds
                            next_bounds.bottom_left.y = if velocity.0.y > 0.0 {
                                collision_pos.y as f32 - next_bounds.width()
                            } else {
                                collision_pos.y as f32 + 1.0
                            };

                            // Update next velocity
                            next_velocity.y = 0.0;

                            (true, next_bounds, next_velocity)
                        },
                    );

                    // Update position and velocity
                    next_bounds = step_bounds;
                    next_velocity = step_velocity;

                    // Don't handle anymore x-axis collisions
                    if collided {
                        break;
                    }
                }

                // Update entity
                *bounds = next_bounds;
                velocity.0 = next_velocity;
            },
        );
    });

    drop(tile_collisions_tx);
    tile_collisions.send_batch(tile_collisions_rx.into_iter());
}

#[instrument(skip(collisions, query))]
pub fn reset_jumps(mut collisions: EventReader<TileCollision>, mut query: Query<&mut JumpStatus>) {
    for collision in collisions.iter() {
        if collision.axis == TileCollisionAxis::Y && collision.entity_velocity.y < 0.0 {
            if let Ok(mut jump_status) = query.get_mut(collision.entity) {
                *jump_status = JumpStatus::OnGround;
            }
        }
    }
}

#[instrument(skip(query))]
pub fn cleanup_kinematics(mut query: Query<&mut Acceleration>) {
    for mut acceleration in query.iter_mut() {
        acceleration.0 = EntityWorldPosition::ZERO;
    }
}

// #[instrument(skip(state, query))]
// pub fn update_transforms(
//     state: Res<PhysicsState>,
//     mut query: Query<(&mut Transform, &EntityWorldRect, &Velocity)>,
// ) {
//     for (mut transform, bounds, velocity) in query.iter_mut() {
//         let center = Vec2::from(bounds.center()).extend(transform.translation[2]);
//         let predicted_offset = Vec2::from(velocity.0).extend(0.0)
//             * state.step_timer.percent()
//             * state.step_timer.duration().as_secs_f32();
//         transform.translation = center + predicted_offset;
//         transform.translation;
//     }
// }

#[instrument(skip(query))]
pub fn update_transforms(mut query: Query<(&mut Transform, &EntityWorldRect)>) {
    for (mut transform, bounds) in query.iter_mut() {
        transform.translation = Vec2::from(bounds.center()).extend(transform.translation[2]);
        transform.translation;
    }
}
