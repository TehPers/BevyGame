use crate::runner::WasmRunner;
use bevy::prelude::*;
use std::path::Path;
use tracing::instrument;

#[instrument(skip(commands))]
pub fn setup_runner(commands: &mut Commands) {
    // TODO: actually load module properly from a directory
    let runner = WasmRunner::new(Path::new("bin/mods")).unwrap();
    commands.insert_resource(runner);
}

#[instrument(skip(runner))]
pub fn on_update(mut runner: ResMut<WasmRunner>) {
    runner.on_update(&[]).unwrap();
}
