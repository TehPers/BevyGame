use bevy::{app::startup_stage, prelude::*};

pub struct WasmPlugin;

impl Plugin for WasmPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system_to_stage(
            startup_stage::PRE_STARTUP,
            crate::systems::setup_runner.system(),
        )
        .add_system_to_stage(stage::UPDATE, crate::systems::on_update.system());
    }
}
