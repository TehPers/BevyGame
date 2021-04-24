use crate::random::GameRandom;
use game_lib::rand::SeedableRng;

#[derive(Clone, Debug, Default)]
pub struct RandomConfig {
    pub seed: Option<<GameRandom as SeedableRng>::Seed>,
}
