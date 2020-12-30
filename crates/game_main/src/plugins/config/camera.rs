use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub enum CameraMode {
    Free,
    FollowPlayer,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CameraConfig {
    pub camera_mode: CameraMode,
}

impl Default for CameraConfig {
    fn default() -> Self {
        CameraConfig {
            camera_mode: CameraMode::Free,
        }
    }
}
