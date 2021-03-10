use bevy::prelude::*;

use crate::Player;

pub struct ControllerPlugin;

impl Plugin for ControllerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.register_type::<Player>()
            .add_system_to_stage(stage::UPDATE, crate::systems::cycle_camera_mode.system())
            .add_system_to_stage(stage::UPDATE, crate::systems::update_camera.system())
            .add_system_to_stage(stage::UPDATE, crate::systems::move_player.system());
    }
}
