use crate::{
    bodies::AxisAlignedBoundingBox, broad_phase::QuadTree, systems, Acceleration, BodyType, Drag,
    EntityCollision, Forces, Gravity, Mass, PhysicsState, TileCollision, Velocity,
};
use bevy::{app::startup_stage, prelude::*};

pub type BroadPhaseQuadTree = QuadTree<Entity, 1, 4, 10>;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.register_type::<Mass>()
            .register_type::<Velocity>()
            .register_type::<Acceleration>()
            .register_type::<Forces>()
            .register_type::<Gravity>()
            .register_type::<Drag>()
            .register_type::<BodyType>()
            .register_type::<AxisAlignedBoundingBox>()
            .register_type::<PhysicsState>()
            .register_type::<EntityCollision>()
            .register_type::<TileCollision>()
            .init_resource::<PhysicsState>()
            .add_event::<EntityCollision>()
            .add_event::<TileCollision>()
            .add_startup_system_to_stage(startup_stage::PRE_STARTUP, systems::setup.system())
            .add_stage_after(
                stage::UPDATE,
                crate::stage::PRE_PHYSICS_STAGE,
                SystemStage::parallel()
                    .with_system(systems::update_bodies.system())
                    .with_system(systems::add_bodies.system())
                    .with_system(systems::update_state.system())
                    .with_system(systems::add_kinematic_forces.system())
                    .with_system(systems::apply_forces.system())
                    .with_system(systems::apply_acceleration.system())
                    .with_system(systems::reset_events.system()),
            )
            .add_stage_after(
                crate::stage::PRE_PHYSICS_STAGE,
                crate::stage::PHYSICS_STAGE,
                SystemStage::serial()
                    .with_run_criteria(systems::should_step.system())
                    .with_system(systems::step.system()),
            )
            .add_stage_after(
                crate::stage::PHYSICS_STAGE,
                crate::stage::POST_PHYSICS_STAGE,
                SystemStage::parallel()
                    .with_system(systems::reset.system())
                    .with_system(systems::update_transforms.system())
                    .with_system(systems::remove_bodies.system()),
            );
    }
}
