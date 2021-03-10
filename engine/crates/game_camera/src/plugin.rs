use crate::{CameraConfig, CameraMode, CameraState, ScaledOrthographicProjection};
use bevy::{app::startup_stage, prelude::*};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.register_type::<CameraState>()
            .register_type::<CameraMode>()
            .register_type::<CameraConfig>()
            .init_resource::<CameraConfig>()
            .add_startup_system_to_stage(startup_stage::STARTUP, crate::systems::setup.system())
            .add_system_to_stage(
                stage::POST_UPDATE,
                crate::systems::camera_projection_changed.system(),
            )
            .add_system_to_stage(
                stage::POST_UPDATE,
                bevy::render::camera::camera_system::<ScaledOrthographicProjection>.system(),
            );
    }
}
