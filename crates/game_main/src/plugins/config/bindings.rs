use crate::plugins::input::ActionInput;
use bevy::{
    prelude::*,
    utils::{AHashExt, HashMap},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InputBindings {
    pub keyboard: HashMap<KeyCode, ActionInput>,
    pub mouse: HashMap<MouseButton, ActionInput>,
}

impl Default for InputBindings {
    fn default() -> Self {
        // Default keyboard bindings
        let mut keyboard = HashMap::new();
        keyboard.insert(KeyCode::Left, ActionInput::CameraLeft);
        keyboard.insert(KeyCode::Right, ActionInput::CameraRight);
        keyboard.insert(KeyCode::Up, ActionInput::CameraUp);
        keyboard.insert(KeyCode::Down, ActionInput::CameraDown);
        keyboard.insert(KeyCode::Equals, ActionInput::CameraIn);
        keyboard.insert(KeyCode::Minus, ActionInput::CameraOut);
        keyboard.insert(KeyCode::C, ActionInput::CycleCameraMode);
        keyboard.insert(KeyCode::A, ActionInput::PlayerLeft);
        keyboard.insert(KeyCode::D, ActionInput::PlayerRight);
        keyboard.insert(KeyCode::Space, ActionInput::PlayerJump);

        // Default mouse bindings
        let mouse = HashMap::new();

        InputBindings { keyboard, mouse }
    }
}
