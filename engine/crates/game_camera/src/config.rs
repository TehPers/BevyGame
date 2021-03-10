use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Reflect)]
pub enum CameraMode {
    Free,
    FollowPlayer,
}

impl Default for CameraMode {
    fn default() -> Self {
        CameraMode::Free
    }
}

#[derive(Debug, Default, Reflect)]
pub struct CameraConfig {
    pub camera_mode: CameraMode,
}
