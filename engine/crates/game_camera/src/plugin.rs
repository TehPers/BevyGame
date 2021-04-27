use crate::{CameraConfig, CameraMode, CameraState, ScaledOrthographicProjection};
use game_core::GameStage;
use game_lib::bevy::{
    ecs as bevy_ecs,
    prelude::*,
    render::{camera::camera_system, RenderSystem},
};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, SystemLabel)]
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.register_type::<CameraState>()
            .register_type::<CameraMode>()
            .register_type::<CameraConfig>()
            .init_resource::<CameraConfig>()
            .add_system_set_to_stage(
                GameStage::Startup,
                SystemSet::new()
                    .label(CameraPlugin)
                    .label(CameraSystem::Setup)
                    .in_ambiguity_set(CameraSystem::Setup)
                    .with_system(crate::systems::setup.system()),
            )
            .add_system_set_to_stage(
                GameStage::GameUpdate,
                SystemSet::new()
                    .label(CameraPlugin)
                    .label(CameraSystem::DetectProjectionChange)
                    .in_ambiguity_set(CameraSystem::DetectProjectionChange)
                    .with_system(crate::systems::camera_projection_changed.system()),
            )
            .add_system_to_stage(
                CoreStage::PostUpdate,
                camera_system::<ScaledOrthographicProjection>
                    .system()
                    .label(CameraPlugin)
                    .before(RenderSystem::VisibleEntities),
            );
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, SystemLabel, AmbiguitySetLabel)]
pub enum CameraSystem {
    Setup,
    DetectProjectionChange,
}
