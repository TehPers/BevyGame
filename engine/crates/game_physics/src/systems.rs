use crate::{
    bodies::AxisAlignedBoundingBox, Acceleration, BodyType, Drag, Forces, Gravity, Mass,
    PhysicsState, TileCollision, Velocity,
};
use game_lib::{
    bevy::{ecs::schedule::ShouldRun, prelude::*, tasks::ComputeTaskPool},
    tracing::{self, instrument},
};
use game_tiles::{
    EntityWorldPosition, EntityWorldRect, GameWorld, Tile, TileWorldPosition, TileWorldRect,
};

#[instrument(skip(commands))]
pub fn setup(mut commands: Commands) {
    commands.insert_resource(PhysicsState::default());
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
    if let Some(queued_steps) = state.queued_steps.checked_shr(1) {
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

                #[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
                enum CollisionAxis {
                    X,
                    Y,
                }

                #[derive(Clone, PartialEq, Debug)]
                struct CollisionData {
                    axis: CollisionAxis,
                    tile: Tile,
                    tile_position: TileWorldPosition,
                }

                // Do tile collisions and calculate next bounds
                let (next_bounds, collision_data) = info_span!("tiles").in_scope(|| {
                    // Calculate next bounds
                    let next_bounds: EntityWorldRect =
                        bounds.offset(velocity.0 * state.step_timer.duration().as_secs_f32());

                    // Get intersecting tiles
                    let intersecting_tiles: TileWorldRect = (*bounds).into();
                    let intersections = intersecting_tiles.iter_positions().flat_map(|position| {
                        world
                            .get_tile(position)
                            // TODO: should the tiles be generated if not already?
                            .ok()
                            .copied()
                            .flatten()
                            .map(|tile| (position, tile))
                    });

                    // Calculate intersection depths
                    let intersection_times = intersections
                        .flat_map(|(position, tile)| {
                            let tile_bounds =
                                AxisAlignedBoundingBox::new(position.into(), Vec2::ONE);

                            // Time (in seconds) to collide along x-axis
                            let x_time = if velocity.0.x > 0.0 {
                                Some(tile_bounds.left() - bounds.right()).filter(|&d| d >= 0.0)
                            } else if velocity.0.x < 0.0 {
                                Some(tile_bounds.right() - bounds.left()).filter(|&d| d <= 0.0)
                            } else {
                                None
                            }
                            .map(|d| {
                                (
                                    d / velocity.0.x,
                                    CollisionData {
                                        axis: CollisionAxis::X,
                                        tile,
                                        tile_position: position,
                                    },
                                )
                            });

                            // Time (in seconds) to collide along y-axis
                            let y_time = if velocity.0.y > 0.0 {
                                Some(tile_bounds.bottom() - bounds.top()).filter(|&d| d >= 0.0)
                            } else if velocity.0.y < 0.0 {
                                Some(tile_bounds.top() - bounds.bottom()).filter(|&d| d <= 0.0)
                            } else {
                                None
                            }
                            .map(|d| {
                                (
                                    d / velocity.0.y,
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
                    intersection_times
                        .min_by(|(a, ..), (b, ..)| a.partial_cmp(b).unwrap())
                        .map(|(t, collision_data)| {
                            (bounds.offset(velocity.0 * t), Some(collision_data))
                        })
                        .unwrap_or((next_bounds, None))
                });

                // Resolve tile collision
                if let Some(collision_data) = collision_data {
                    // Send collision event
                    tile_collisions_tx
                        .send(TileCollision {
                            entity,
                            entity_velocity: velocity.0,
                            tile: collision_data.tile,
                            tile_position: collision_data.tile_position,
                        })
                        .unwrap();

                    // Add friction while standing on a surface
                    const FRICTION_COEFFICIENT: f32 = 0.8;
                    if collision_data.axis == CollisionAxis::Y && velocity.0.y < 0.0 {
                        // simplified friction, normal friction is F = mu * N
                        velocity.0.x *= FRICTION_COEFFICIENT;
                    }

                    // Adjust velocity due to tile collision
                    match collision_data.axis {
                        CollisionAxis::X => velocity.0.x = 0.0,
                        CollisionAxis::Y => velocity.0.y = 0.0,
                    }
                }

                // Update entity position
                *bounds = next_bounds;
            },
        );
    });

    drop(tile_collisions_tx);
    tile_collisions.send_batch(tile_collisions_rx.into_iter());
}

// #[instrument(skip(
//     pool,
//     quadtree,
//     state,
//     tile_collisions,
//     entity_collisions,
//     world,
//     query
// ))]
// pub fn step(
//     pool: Res<ComputeTaskPool>,
//     quadtree: Res<BroadPhaseQuadTree>,
//     mut state: ResMut<PhysicsState>,
//     mut tile_collisions: EventWriter<TileCollision>,
//     mut entity_collisions: EventWriter<EntityCollision>,
//     world: Res<GameWorld>,
//     mut query: Query<(
//         Entity,
//         &mut AxisAlignedBoundingBox,
//         &mut Velocity,
//         &BodyType,
//     )>,
// ) {
//     let state = &mut *state;

//     // Decrement queued steps
//     if let Some(queued_steps) = state.queued_steps.checked_shr(1) {
//         state.queued_steps = queued_steps;
//     }

//     #[derive(Clone, Debug)]
//     enum Collision {
//         Entity(EntityCollision),
//         Tile(TileCollision),
//     }

//     let (tx, rx) = game_lib::crossbeam::channel::unbounded();
//     let quadtree = &*quadtree;
//     let state = &*state;
//     let world = &*world;
//     info_span!("collisions").in_scope(|| {
//         query.par_for_each_mut(
//             &pool,
//             25,
//             move |(entity, mut bounds, mut velocity, &body_type)| {
//                 // Only step on kinematic bodies
//                 if body_type != BodyType::Kinematic {
//                     return;
//                 }

//                 #[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
//                 enum CollisionAxis {
//                     X,
//                     Y,
//                 }

//                 #[derive(Clone, PartialEq, Debug)]
//                 struct CollisionData {
//                     axis: CollisionAxis,
//                     tile: Tile,
//                     tile_position: TileWorldPosition,
//                 }

//                 // Do tile collisions and calculate next bounds
//                 let (next_bounds, collision_data) = info_span!("tiles").in_scope(|| {
//                     // Calculate next bounds
//                     let next_bounds: AxisAlignedBoundingBox =
//                         *bounds + velocity.0 * state.step_timer.duration().as_secs_f32();

//                     // Get intersecting tiles
//                     // let intersections = world
//                     //     .iter_intersecting(next_bounds.bottom_left(), next_bounds.top_right())
//                     //     .flat_map(|(position, result)| {
//                     //         match result {
//                     //             Ok(tile) => tile,
//                     //             Err(GetTileError::OutOfBounds(..)) => None,
//                     //             Err(error) => {
//                     //                 error!(
//                     //                     ?error,
//                     //                     "failure getting tile for collision checking: {}", error
//                     //                 );
//                     //                 None
//                     //             }
//                     //         }
//                     //         .map(move |tile| (position, tile))
//                     //     });

//                     // Calculate intersection depths
//                     // let intersection_times = intersections
//                     //     .flat_map(|(position, tile)| {
//                     //         let tile_bounds =
//                     //             AxisAlignedBoundingBox::new(position.into(), Vec2::ONE);

//                     //         // Time (in seconds) to collide along x-axis
//                     //         let x_time = if velocity.0[0] > 0.0 {
//                     //             Some(tile_bounds.left() - bounds.right()).filter(|&d| d >= 0.0)
//                     //         } else if velocity.0[0] < 0.0 {
//                     //             Some(tile_bounds.right() - bounds.left()).filter(|&d| d <= 0.0)
//                     //         } else {
//                     //             None
//                     //         }
//                     //         .map(|d| {
//                     //             (
//                     //                 d / velocity.0[0],
//                     //                 CollisionData {
//                     //                     axis: CollisionAxis::X,
//                     //                     tile,
//                     //                     tile_position: position,
//                     //                 },
//                     //             )
//                     //         });

//                     //         // Time (in seconds) to collide along y-axis
//                     //         let y_time = if velocity.0[1] > 0.0 {
//                     //             Some(tile_bounds.bottom() - bounds.top()).filter(|&d| d >= 0.0)
//                     //         } else if velocity.0[1] < 0.0 {
//                     //             Some(tile_bounds.top() - bounds.bottom()).filter(|&d| d <= 0.0)
//                     //         } else {
//                     //             None
//                     //         }
//                     //         .map(|d| {
//                     //             (
//                     //                 d / velocity.0[1],
//                     //                 CollisionData {
//                     //                     axis: CollisionAxis::Y,
//                     //                     tile,
//                     //                     tile_position: position,
//                     //                 },
//                     //             )
//                     //         });

//                     //         std::iter::once(x_time)
//                     //             .chain(std::iter::once(y_time))
//                     //             .flatten()
//                     //     })
//                     //     .filter(|(t, ..)| t.is_finite());

//                     // Calculate minimum intersection time (first tile collision)
//                     // let min_time =
//                     //     intersection_times.min_by(|(a, ..), (b, ..)| a.partial_cmp(b).unwrap());
//                     // min_time
//                     //     .map(|(t, collision_data)| (*bounds + velocity.0 * t, Some(collision_data)))
//                     //     .unwrap_or((next_bounds, None))
//                     (next_bounds, Option::<CollisionData>::None)
//                 });

//                 // Resolve tile collision
//                 if let Some(collision_data) = collision_data {
//                     // Send collision event
//                     tx.send(Collision::Tile(TileCollision {
//                         entity,
//                         entity_velocity: velocity.0,
//                         tile: collision_data.tile,
//                         tile_position: collision_data.tile_position,
//                     }))
//                     .unwrap();

//                     // Add friction while standing on a surface
//                     const FRICTION_COEFFICIENT: f32 = 0.8;
//                     if collision_data.axis == CollisionAxis::Y && velocity.0[1] < 0.0 {
//                         // simplified friction, normal friction is F = mu * N
//                         velocity.0[0] *= FRICTION_COEFFICIENT;
//                     }

//                     // Adjust velocity due to tile collision
//                     match collision_data.axis {
//                         CollisionAxis::X => velocity.0[0] = 0.0,
//                         CollisionAxis::Y => velocity.0[1] = 0.0,
//                     }
//                 }

//                 // Update entity position
//                 *bounds = next_bounds;

//                 let bounds = *bounds;
//                 info_span!("entities").in_scope(|| {
//                     // Broad phase
//                     let broad_collisions = quadtree.query_bounds(bounds);

//                     // Narrow phase
//                     let narrow_collisions = broad_collisions
//                         .filter(|&entry| {
//                             quadtree
//                                 .get_bounds(entry)
//                                 .filter(|other| bounds.intersects(other))
//                                 .is_some()
//                         })
//                         .filter_map(|entry| quadtree.get(entry).copied());

//                     for other_entity in narrow_collisions {
//                         tx.send(Collision::Entity(EntityCollision {
//                             entities: (entity, other_entity),
//                         }))
//                         .unwrap();
//                     }
//                 });
//             },
//         );
//     });

//     info_span!("process_collisions").in_scope(|| {
//         for event in rx {
//             match event {
//                 Collision::Tile(event) => tile_collisions.send(event),
//                 Collision::Entity(event) => entity_collisions.send(event),
//             }
//         }
//     });
// }

#[instrument(skip(query))]
pub fn cleanup_kinematics(mut query: Query<&mut Acceleration>) {
    for mut acceleration in query.iter_mut() {
        acceleration.0 = EntityWorldPosition::ZERO;
    }
}

#[instrument(skip(state, query))]
pub fn update_transforms(
    state: Res<PhysicsState>,
    mut query: Query<(&mut Transform, &AxisAlignedBoundingBox, &Velocity)>,
) {
    for (mut transform, aabb, velocity) in query.iter_mut() {
        let center = aabb.center().extend(transform.translation[2]);
        let predicted_offset = Vec2::from(velocity.0).extend(0.0)
            * state.step_timer.percent()
            * state.step_timer.duration().as_secs_f32();
        transform.translation = center + predicted_offset;
        transform.translation;
    }
}
