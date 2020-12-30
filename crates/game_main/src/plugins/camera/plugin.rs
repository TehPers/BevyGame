use crate::plugins::{
    camera::{ScaledCamera2dBundle, ScaledOrthographicProjection},
    config::{CameraConfig, CameraMode},
    input::ActionInput,
    Player,
};
use bevy::{
    app::startup_stage,
    prelude::*,
    render::camera::{Camera, CameraProjection},
};
use tracing::instrument;

#[derive(Debug)]
pub struct CameraState {
    pub main_camera: Entity,
}

pub struct CameraPlugin;

impl CameraPlugin {
    #[instrument(skip(commands))]
    fn setup(commands: &mut Commands) {
        let main_camera = commands
            .spawn(ScaledCamera2dBundle {
                orthographic_projection: ScaledOrthographicProjection {
                    zoom: 32.0,
                    ..Default::default()
                },
                ..Default::default()
            })
            .current_entity()
            .unwrap();

        commands.insert_resource(CameraState { main_camera });
    }

    #[instrument(skip(windows, query))]
    fn camera_projection_changed(
        windows: Res<Windows>,
        mut query: Query<
            (&mut Camera, &mut ScaledOrthographicProjection),
            Changed<ScaledOrthographicProjection>,
        >,
    ) {
        for (mut camera, mut camera_projection) in query.iter_mut() {
            if let Some(window) = windows.get(camera.window) {
                camera_projection.update(window.width(), window.height());
                camera.projection_matrix = camera_projection.get_projection_matrix();
                camera.depth_calculation = camera_projection.depth_calculation();
            }
        }
    }

    #[instrument(skip(config, input))]
    fn cycle_camera_mode(mut config: ResMut<CameraConfig>, input: Res<Input<ActionInput>>) {
        if input.just_released(ActionInput::CycleCameraMode) {
            config.camera_mode = match config.camera_mode {
                CameraMode::FollowPlayer => CameraMode::Free,
                CameraMode::Free => CameraMode::FollowPlayer,
            }
        }
    }

    #[instrument(skip(config, input, player_query, time, camera_query))]
    fn update_camera(
        config: Res<CameraConfig>,
        input: Res<Input<ActionInput>>,
        player_query: Query<&Transform, With<Player>>,
        time: Res<Time>,
        mut camera_query: Query<(&mut Transform, &mut ScaledOrthographicProjection), With<Camera>>,
    ) {
        match config.camera_mode {
            CameraMode::FollowPlayer => {
                let player_pos = player_query.iter().nth(0).unwrap().translation;
                for (mut camera, _) in camera_query.iter_mut() {
                    camera.translation = player_pos;
                }
            }
            CameraMode::Free => {
                // Get direction to move
                let mut direction = Vec3::default();
                if input.pressed(ActionInput::CameraUp) {
                    direction += Vec3::unit_y();
                }
                if input.pressed(ActionInput::CameraDown) {
                    direction -= Vec3::unit_y();
                }
                if input.pressed(ActionInput::CameraLeft) {
                    direction -= Vec3::unit_x();
                }
                if input.pressed(ActionInput::CameraRight) {
                    direction += Vec3::unit_x();
                }

                // Get amount to zoom in/out
                let mut zoom_direction = 0.0;
                if input.pressed(ActionInput::CameraIn) {
                    zoom_direction += 1.0;
                }
                if input.pressed(ActionInput::CameraOut) {
                    zoom_direction -= 1.0;
                }

                for (mut camera, mut projection) in camera_query.iter_mut() {
                    // Translate camera if needed
                    let velocity = 1000.0 * time.delta_seconds() / projection.zoom;
                    let offset = direction * velocity;
                    if offset.length_squared() > velocity / 10.0 {
                        camera.translation += offset;
                    }

                    // Adjust camera zoom if needed
                    let zoom_velocity = 32.0 * time.delta_seconds();
                    let zoom_offset = zoom_direction * zoom_velocity;
                    let new_zoom = (projection.zoom + zoom_offset).max(0.1);
                    if (projection.zoom - new_zoom).abs() > 0.001 {
                        projection.zoom = new_zoom;
                    }
                }
            }
        }
    }
}

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system_to_stage(startup_stage::STARTUP, Self::setup.system())
            .add_system_to_stage(stage::UPDATE, Self::cycle_camera_mode.system())
            .add_system_to_stage(stage::UPDATE, Self::update_camera.system())
            .add_system_to_stage(stage::POST_UPDATE, Self::camera_projection_changed.system())
            .add_system_to_stage(
                stage::POST_UPDATE,
                bevy::render::camera::camera_system::<ScaledOrthographicProjection>.system(),
            );
    }
}
