use crate::plugins::config::DebugConfig;
use bevy::prelude::*;

pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.register_type::<DebugConfig>()
            .init_resource::<DebugConfig>();
    }
}
