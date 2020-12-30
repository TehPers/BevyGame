#![feature(duration_zero, duration_saturating_ops)]
#![allow(dead_code)]

mod plugins;
mod util;

use bevy::{
    diagnostic::{DiagnosticsPlugin, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use plugins::physics::{Drag, Gravity, PhysicsState};

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
        .add_plugin(crate::plugins::RandomPlugin)
        .add_plugin(crate::plugins::CameraPlugin)
        .add_plugin(crate::plugins::PhysicsPlugin)
        .add_plugin(crate::plugins::InputPlugin)
        .add_plugin(crate::plugins::PlayerPlugin)
        .add_plugin(crate::plugins::TilesPlugin)
        .add_plugin(crate::plugins::ConfigPlugin)
        .add_plugin(crate::plugins::DebugPlugin)
        .add_plugin(crate::plugins::TimedPlugin)
        .add_resource(PhysicsState {
            drag: Drag::from_terminal_velocity(10.0, 62.0, 9.81),
            gravity: Gravity(Vec2::unit_y() * -9.81),
            // gravity: Gravity(Vec2::zero()),
            ..Default::default()
        })
        .run();
}
