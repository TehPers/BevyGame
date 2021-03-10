use bevy::prelude::*;

#[derive(Debug, Reflect)]
pub struct DebugConfig {
    pub enable_teleporting: bool,
    pub show_quadtree: bool,
}

impl Default for DebugConfig {
    fn default() -> Self {
        DebugConfig {
            enable_teleporting: false,
            show_quadtree: false,
        }
    }
}
