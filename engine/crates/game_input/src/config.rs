use crate::ActionInput;
use game_lib::bevy::{prelude::*, utils::HashMap};

#[derive(Clone, Debug)]
// TODO: once bevy supports reflection for inputs, uncomment this:
// #[derive(Reflect)]
pub struct InputBindings {
    pub keyboard: HashMap<KeyCode, ActionInput>,
    pub mouse: HashMap<MouseButton, ActionInput>,
}

impl Default for InputBindings {
    fn default() -> Self {
        // Default keyboard bindings
        let mut keyboard = HashMap::default();
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
        let mouse = HashMap::default();

        InputBindings { keyboard, mouse }
    }
}
