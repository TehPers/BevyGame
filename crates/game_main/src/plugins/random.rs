use bevy::{app::startup_stage, prelude::*};
use rand::SeedableRng;
use rand_pcg::Pcg64Mcg;
use tracing::instrument;

pub type Random = Pcg64Mcg;

#[derive(Default)]
pub struct RandomConfig {
    seed: Option<<Random as SeedableRng>::Seed>,
}

pub struct RandomPlugin;

impl RandomPlugin {
    #[instrument(skip(commands, config))]
    fn setup(commands: &mut Commands, config: Res<RandomConfig>) {
        let random = match config.seed {
            Some(seed) => Random::from_seed(seed),
            None => Random::from_entropy(),
        };
        commands.insert_resource(random);
    }
}

impl Plugin for RandomPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<RandomConfig>()
            .add_startup_system_to_stage(startup_stage::PRE_STARTUP, Self::setup.system());
    }
}
