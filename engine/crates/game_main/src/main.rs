pub use game_camera as camera;
pub use game_controller as controller;
pub use game_core as core;
pub use game_input as input;
pub use game_physics as physics;
pub use game_tiles as tiles;
pub use game_wasi as wasi;

mod plugins;

use game_lib::bevy::{
    diagnostic::{DiagnosticsPlugin, FrameTimeDiagnosticsPlugin},
    ecs::schedule::ReportExecutionOrderAmbiguities,
    prelude::*,
};
use physics::{Drag, Gravity, PhysicsState};
use tiles::EntityWorldPosition;

game_lib::fix_bevy_derive!(game_lib::bevy);

#[bevy_main]
fn main() {
    App::build()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(WindowDescriptor {
            title: "Tiles".into(),
            ..Default::default()
        })
        .insert_resource(ReportExecutionOrderAmbiguities)
        .add_plugins(DefaultPlugins)
        .add_plugin(FrameTimeDiagnosticsPlugin)
        .add_plugin(DiagnosticsPlugin)
        .add_plugin(crate::core::CorePlugin)
        .add_plugin(crate::core::random::RandomPlugin)
        .add_plugin(crate::tiles::TilePlugin)
        .add_plugin(crate::camera::CameraPlugin)
        .add_plugin(crate::physics::PhysicsPlugin)
        .add_plugin(crate::input::InputPlugin)
        .add_plugin(crate::controller::ControllerPlugin)
        .add_plugin(crate::plugins::PlayerPlugin)
        .add_plugin(crate::plugins::ConfigPlugin)
        .add_plugin(crate::plugins::DebugPlugin)
        .add_plugin(crate::plugins::TimedPlugin)
        // .add_plugin(crate::wasi::WasmPlugin)
        .insert_resource(PhysicsState {
            drag: Drag::from_terminal_velocity(10.0, 62.0, 9.81),
            gravity: Gravity(EntityWorldPosition::Y * -9.81),
            // gravity: Gravity(EntityWorldPosition::ZERO),
            ..Default::default()
        })
        .run();
}
