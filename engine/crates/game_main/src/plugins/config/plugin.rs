use crate::plugins::config::DebugConfig;
use game_lib::bevy::{ecs as bevy_ecs, prelude::*};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, SystemLabel)]
pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.register_type::<DebugConfig>()
            .init_resource::<DebugConfig>();
    }
}
