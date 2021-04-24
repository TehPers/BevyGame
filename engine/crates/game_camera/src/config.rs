use game_lib::bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Reflect)]
pub enum CameraMode {
    Free,
    FollowPlayer,
}

impl Default for CameraMode {
    fn default() -> Self {
        if cfg!(debug) {
            CameraMode::Free
        } else {
            CameraMode::FollowPlayer
        }
    }
}

#[derive(Debug, Default, Reflect)]
pub struct CameraConfig {
    pub camera_mode: CameraMode,
}
