use crate::plugins::camera::ScaledOrthographicProjection;
use bevy::{
    prelude::*,
    render::{
        camera::{Camera, VisibleEntities},
        render_graph::base::camera,
    },
};

#[derive(Bundle)]
pub struct ScaledCamera2dBundle {
    pub camera: Camera,
    pub orthographic_projection: ScaledOrthographicProjection,
    pub visible_entities: VisibleEntities,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl Default for ScaledCamera2dBundle {
    fn default() -> Self {
        let far = 1000.0;
        ScaledCamera2dBundle {
            camera: Camera {
                name: Some(camera::CAMERA_2D.to_string()),
                ..Default::default()
            },
            orthographic_projection: ScaledOrthographicProjection {
                far,
                ..Default::default()
            },
            visible_entities: Default::default(),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, far - 0.1)),
            global_transform: Default::default(),
        }
    }
}
