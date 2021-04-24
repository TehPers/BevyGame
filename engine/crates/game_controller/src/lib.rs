pub(crate) mod systems;

mod entity;
mod plugin;

pub use entity::*;
pub use plugin::*;

game_lib::fix_bevy_derive!(game_lib::bevy);
