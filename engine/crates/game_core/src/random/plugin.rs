use crate::random::{RandomConfig, ResetRandom};
use bevy::{app::startup_stage, prelude::*};
use rand_pcg::Pcg64Mcg;

pub type GameRandom = Pcg64Mcg;

pub struct RandomPlugin;

impl Plugin for RandomPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.register_type::<RandomConfig>()
            .register_type::<ResetRandom>()
            .add_event::<ResetRandom>()
            .add_startup_system_to_stage(
                startup_stage::PRE_STARTUP,
                crate::random::systems::setup.system(),
            )
            .add_system_to_stage(stage::EVENT, crate::random::systems::reset_random.system());

        let resources = app.resources_mut();
        if resources.get::<RandomConfig>().is_none() {
            resources.insert(RandomConfig::default());
        }
    }
}
