#![feature(duration_zero, duration_saturating_ops)]
#![allow(dead_code)]

pub use game_camera as camera;
pub use game_controller as controller;
pub use game_core as core;
pub use game_input as input;
pub use game_physics as physics;
pub use game_tiles as tiles;
pub use game_wasi as wasi;

mod plugins;

use bevy::{
    diagnostic::{DiagnosticsPlugin, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use physics::{Drag, Gravity, PhysicsState};

#[bevy_main]
fn main() {
    App::build()
        .add_resource(ClearColor(Color::BLACK))
        .add_resource(WindowDescriptor {
            title: "Tiles".into(),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(FrameTimeDiagnosticsPlugin)
        .add_plugin(DiagnosticsPlugin)
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
        .add_resource(PhysicsState {
            drag: Drag::from_terminal_velocity(10.0, 62.0, 9.81),
            gravity: Gravity(Vec2::unit_y() * -9.81),
            // gravity: Gravity(Vec2::zero()),
            ..Default::default()
        })
        .run();
}
