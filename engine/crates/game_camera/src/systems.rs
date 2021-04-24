use crate::{CameraState, ScaledCamera2dBundle, ScaledOrthographicProjection};
use game_lib::{
    bevy::{
        prelude::*,
        render::camera::{Camera, CameraProjection},
    },
    tracing::{self, instrument},
};

#[instrument(skip(commands))]
pub fn setup(mut commands: Commands) {
    let ui_camera = commands.spawn_bundle(UiCameraBundle::default()).id();
    let main_camera = commands
        .spawn_bundle(ScaledCamera2dBundle {
            orthographic_projection: ScaledOrthographicProjection {
                zoom: 32.0,
                ..Default::default()
            },
            ..Default::default()
        })
        .id();

    commands.insert_resource(CameraState {
        main_camera,
        ui_camera,
    });
}

#[instrument(skip(windows, query))]
pub fn camera_projection_changed(
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
