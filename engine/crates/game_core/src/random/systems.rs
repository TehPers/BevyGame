use crate::random::{GameRandom, RandomConfig, ResetRandom};
use game_lib::{
    bevy::prelude::*,
    rand::SeedableRng,
    tracing::{self, instrument},
};

#[instrument(skip(commands, config))]
pub fn setup(mut commands: Commands, config: Res<RandomConfig>) {
    let random = config
        .seed
        .map(|seed| GameRandom::from_seed(seed))
        .unwrap_or_else(|| GameRandom::from_entropy());

    commands.insert_resource(random);
}

#[instrument(skip(reset_event, config, random))]
pub fn reset_random(
    mut reset_event: EventReader<ResetRandom>,
    config: Res<RandomConfig>,
    mut random: ResMut<GameRandom>,
) {
    if reset_event.iter().last().is_some() {
        let new_random = config
            .seed
            .map(|seed| GameRandom::from_seed(seed))
            .unwrap_or_else(|| GameRandom::from_entropy());

        *random = new_random;
    }
}
