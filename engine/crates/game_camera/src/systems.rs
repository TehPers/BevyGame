use crate::{CameraState, ScaledCamera2dBundle, ScaledOrthographicProjection};
use bevy::{
    prelude::*,
    render::camera::{Camera, CameraProjection},
};
use tracing::instrument;

#[instrument(skip(commands))]
pub fn setup(commands: &mut Commands) {
    let ui_camera = commands
        .spawn(CameraUiBundle::default())
        .current_entity()
        .unwrap();

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
