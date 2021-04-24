pub mod generation;
pub mod render;
pub(crate) mod systems;

mod plugin;
mod tile;
mod world;

pub use plugin::*;
pub use tile::*;
pub use world::*;

game_lib::fix_bevy_derive!(game_lib::bevy);
