use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub enum ActionInput {
    CameraUp,
    CameraDown,
    CameraRight,
    CameraLeft,
    CameraIn,
    CameraOut,
    CycleCameraMode,
    PlayerJump,
    PlayerLeft,
    PlayerRight,
}
