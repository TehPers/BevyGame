use crate::plugins::config::{CameraConfig, DebugConfig, InputBindings};
use bevy::prelude::*;

pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<CameraConfig>()
            .init_resource::<DebugConfig>()
            .init_resource::<InputBindings>();
    }
}
