use crate::{
    bodies::AxisAlignedBoundingBox, systems, Acceleration, BodyType, Drag, EntityCollision, Forces,
    Gravity, Mass, PhysicsState, TileCollision, Velocity,
};
use game_core::{combinators::if_all, modes::ModeExt, GameStage, GlobalMode, ModeEvent};
use game_lib::bevy::{ecs as bevy_ecs, prelude::*};
use game_tiles::TileSystem;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, SystemLabel)]
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
            .add_event::<EntityCollision>()
            .add_event::<TileCollision>()
            .add_system_set_to_stage(
                GameStage::PreUpdate,
                SystemSet::new()
                    .label(PhysicsPlugin)
                    .with_run_criteria(GlobalMode::InGame.on(ModeEvent::Enter))
                    .with_system(systems::setup.system()),
            )
            .add_system_set_to_stage(
                GameStage::PostUpdate,
                SystemSet::new()
                    .label(PhysicsPlugin)
                    .with_run_criteria(GlobalMode::InGame.on(ModeEvent::Exit))
                    .with_system(systems::cleanup.system()),
            )
            .add_system_set_to_stage(
                GameStage::Update,
                SystemSet::new()
                    .label(PhysicsPlugin)
                    .label(PhysicsSystem::UpdateState)
                    .with_run_criteria(GlobalMode::InGame.on(ModeEvent::Active))
                    .with_system(systems::update_physics_state.system()),
            )
            .add_system_set_to_stage(
                GameStage::Update,
                SystemSet::new()
                    .label(PhysicsPlugin)
                    .label(PhysicsSystem::Prepare)
                    .after(PhysicsSystem::UpdateState)
                    .with_run_criteria(if_all(vec![
                        GlobalMode::InGame.on(ModeEvent::Active),
                        Box::new(systems::if_physics_lagged.system()),
                    ]))
                    .with_system(
                        systems::add_kinematic_forces
                            .system()
                            .chain(systems::apply_forces.system())
                            .chain(systems::apply_acceleration.system()),
                    ),
            )
            .add_system_set_to_stage(
                GameStage::Update,
                SystemSet::new()
                    .label(PhysicsPlugin)
                    .label(PhysicsSystem::Run)
                    .after(PhysicsSystem::Prepare)
                    .with_run_criteria(if_all(vec![
                        GlobalMode::InGame.on(ModeEvent::Active),
                        Box::new(systems::while_physics_lagged.system()),
                    ]))
                    .with_system(systems::step.system()),
            )
            .add_system_set_to_stage(
                GameStage::Update,
                SystemSet::new()
                    .label(PhysicsPlugin)
                    .label(PhysicsSystem::Cleanup)
                    .before(TileSystem::DetectRedraw)
                    .after(PhysicsSystem::Run)
                    .with_run_criteria(GlobalMode::InGame.on(ModeEvent::Active))
                    .with_system(systems::update_transforms.system()),
            )
            .add_system_set_to_stage(
                GameStage::Update,
                SystemSet::new()
                    .label(PhysicsPlugin)
                    .label(PhysicsSystem::Cleanup)
                    .after(PhysicsSystem::Run)
                    .with_run_criteria(if_all(vec![
                        GlobalMode::InGame.on(ModeEvent::Active),
                        Box::new(systems::if_physics_lagged.system()),
                    ]))
                    .with_system(systems::cleanup_kinematics.system()),
            );
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, SystemLabel)]
pub enum PhysicsSystem {
    UpdateState,
    Prepare,
    Run,
    Cleanup,
}
