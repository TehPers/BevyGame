use crate::runner::WasmRunner;
use game_lib::{
    bevy::prelude::*,
    tracing::{self, instrument},
};
use std::path::Path;

#[instrument(skip(commands))]
pub fn setup_runner(mut commands: Commands) {
    // TODO: actually load module properly from a directory
    let runner = WasmRunner::new(Path::new("bin/mods")).unwrap();
    commands.insert_resource(runner);
}

#[instrument(skip(runner))]
pub fn on_update(mut runner: ResMut<WasmRunner>) {
    runner.on_update(&[]).unwrap();
}
