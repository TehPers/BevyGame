pub mod bodies;
pub mod broad_phase;
pub mod narrow_phase;

mod components;
mod events;
mod line;
mod plugin;
mod resources;
mod systems;

pub use components::*;
pub use events::*;
pub use line::*;
pub use plugin::*;
pub use resources::*;

game_lib::fix_bevy_derive!(game_lib::bevy);
