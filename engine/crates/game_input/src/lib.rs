pub(crate) mod systems;

mod action;
mod config;
mod cursor;
mod plugin;

pub use action::*;
pub use config::*;
pub use cursor::*;
pub use plugin::*;

game_lib::fix_bevy_derive!(game_lib::bevy);
