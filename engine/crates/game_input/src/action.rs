use game_lib::bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Reflect)]
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
