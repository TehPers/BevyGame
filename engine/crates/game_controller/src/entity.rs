use bevy::prelude::*;

#[derive(Debug, Default, Bundle)]
pub struct PlayerBundle {
    player: Player,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash, Reflect)]
pub struct Player;
