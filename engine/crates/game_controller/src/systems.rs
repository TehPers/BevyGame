use crate::Player;
use game_camera::{CameraConfig, CameraMode, ScaledOrthographicProjection};
use game_input::ActionInput;
use game_lib::{
    bevy::{prelude::*, render::camera::Camera},
    tracing::{self, instrument},
};
use game_physics::Velocity;

#[instrument(skip(config, input))]
pub fn cycle_camera_mode(mut config: ResMut<CameraConfig>, input: Res<Input<ActionInput>>) {
    if input.just_released(ActionInput::CycleCameraMode) {
        config.camera_mode = match config.camera_mode {
            CameraMode::FollowPlayer => CameraMode::Free,
            CameraMode::Free => CameraMode::FollowPlayer,
        }
    }
}

#[instrument(skip(config, input, time, player_query, camera_query))]
pub fn update_camera(
    config: Res<CameraConfig>,
    input: Res<Input<ActionInput>>,
    time: Res<Time>,
    player_query: Query<&Transform, (With<Player>, Changed<Transform>)>,
    mut camera_query: Query<
        (&mut Transform, &mut ScaledOrthographicProjection),
        (Without<Player>, With<Camera>),
    >,
) {
    match config.camera_mode {
        CameraMode::FollowPlayer => {
            if let Ok(player_transform) = player_query.single() {
                let player_pos = player_transform.translation;
                for (mut camera, _) in camera_query.iter_mut() {
                    camera.translation = player_pos;
                }
            }
        }
        CameraMode::Free => {
            // Get direction to move
            let mut direction = Vec3::default();
            if input.pressed(ActionInput::CameraUp) {
                direction += Vec3::Y;
            }
            if input.pressed(ActionInput::CameraDown) {
                direction -= Vec3::Y;
            }
            if input.pressed(ActionInput::CameraLeft) {
                direction -= Vec3::X;
            }
            if input.pressed(ActionInput::CameraRight) {
                direction += Vec3::X;
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

#[instrument(skip(input, query))]
pub fn move_player(input: Res<Input<ActionInput>>, mut query: Query<&mut Velocity, With<Player>>) {
    // Get direction to move
    const MOVE_SPEED: f32 = 5.0;
    let mut move_velocity = Vec2::default();
    if input.pressed(ActionInput::PlayerLeft) {
        move_velocity -= Vec2::X * MOVE_SPEED;
    }
    if input.pressed(ActionInput::PlayerRight) {
        move_velocity += Vec2::X * MOVE_SPEED;
    }

    // Apply force
    for mut velocity in query.iter_mut() {
        if move_velocity.length_squared() >= 0.1 {
            if move_velocity.x > 0.0 && move_velocity.x > velocity.0.x {
                velocity.0.x = (velocity.0.x + move_velocity.x).min(move_velocity.x);
            } else if move_velocity.x < 0.0 && move_velocity.x < velocity.0.x {
                velocity.0.x = (velocity.0.x + move_velocity.x).max(move_velocity.x);
            }
        }
    }
}
