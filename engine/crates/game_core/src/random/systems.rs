use crate::random::{GameRandom, RandomConfig, ResetRandom};
use bevy::prelude::*;
use rand::SeedableRng;
use tracing::instrument;

#[instrument(skip(commands))]
pub fn setup(commands: &mut Commands, config: Res<RandomConfig>) {
    let random = config
        .seed
        .map(|seed| GameRandom::from_seed(seed))
        .unwrap_or_else(|| GameRandom::from_entropy());

    commands.insert_resource(random);
}

#[instrument(skip(event, event_reader, config, random))]
pub fn reset_random(
    event: Res<Events<ResetRandom>>,
    mut event_reader: Local<EventReader<ResetRandom>>,
    config: Res<RandomConfig>,
    mut random: ResMut<GameRandom>,
) {
    if event_reader.latest(&event).is_some() {
        let new_random = config
            .seed
            .map(|seed| GameRandom::from_seed(seed))
            .unwrap_or_else(|| GameRandom::from_entropy());

        *random = new_random;
    }
}
