use game_lib::bevy::prelude::*;

#[derive(Debug, Reflect)]
pub struct CameraState {
    pub main_camera: Entity,
    pub ui_camera: Entity,
}
