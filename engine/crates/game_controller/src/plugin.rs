use crate::Player;
use game_camera::CameraSystem;
use game_core::{GameStage, GlobalMode, ModeEvent, modes::ModeExt};
use game_lib::bevy::{ecs as bevy_ecs, prelude::*};
use game_physics::PhysicsPlugin;
use game_tiles::TileSystem;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, SystemLabel)]
pub struct ControllerPlugin;

impl Plugin for ControllerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.register_type::<Player>()
            .add_system_set_to_stage(
                GameStage::GameUpdate,
                SystemSet::new()
                    .label(ControllerPlugin)
                    .label(ControllerSystem::UpdateConfig)
                    .with_system(crate::systems::cycle_camera_mode.system()),
            )
            .add_system_set_to_stage(
                GameStage::GameUpdate,
                SystemSet::new()
                    .label(ControllerPlugin)
                    .label(ControllerSystem::HandleControls)
                    .after(ControllerSystem::UpdateConfig)
                    .with_run_criteria(GlobalMode::InGame.on(ModeEvent::Active))
                    .with_system(crate::systems::move_player.system()),
            )
            .add_system_set_to_stage(
                GameStage::GameUpdate,
                SystemSet::new()
                    .label(ControllerPlugin)
                    .label(ControllerSystem::UpdateCamera)
                    .before(CameraSystem::DetectProjectionChange)
                    .before(TileSystem::DetectRedraw)
                    .after(PhysicsPlugin)
                    .with_system(crate::systems::update_camera.system()),
            );
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, SystemLabel)]
pub enum ControllerSystem {
    UpdateConfig,
    HandleControls,
    UpdateCamera,
}
