use crate::random::GameRandom;
use bevy::prelude::*;
use rand::SeedableRng;

#[derive(Clone, Debug, Default, Reflect)]
pub struct RandomConfig {
    pub seed: Option<<GameRandom as SeedableRng>::Seed>,
}
