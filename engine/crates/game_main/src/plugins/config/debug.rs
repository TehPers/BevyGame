use game_lib::bevy::prelude::*;

#[derive(Debug, Reflect)]
pub struct DebugConfig {
    pub enable_teleporting: bool,
}

impl Default for DebugConfig {
    fn default() -> Self {
        DebugConfig {
            enable_teleporting: false,
        }
    }
}
