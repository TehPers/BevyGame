use game_core::GameStage;
use game_lib::bevy::{ecs as bevy_ecs, prelude::*};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, SystemLabel)]
pub struct WasmPlugin;

impl Plugin for WasmPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_set_to_stage(
            GameStage::Startup,
            SystemSet::new().with_system(crate::systems::setup_runner.system()),
        )
        .add_system_to_stage(GameStage::GameUpdate, crate::systems::on_update.system());
    }
}
