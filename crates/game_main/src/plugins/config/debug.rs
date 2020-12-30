use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
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
