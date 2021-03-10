use bevy::prelude::*;

/// Resets the state of the RNG. This event is useful for situations where the
/// seed changes, the world is reloaded, etc.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash, Reflect)]
pub struct ResetRandom;
