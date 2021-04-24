use game_lib::bevy::{ecs as bevy_ecs, prelude::*};
use game_physics::PhysicsBundle;

#[derive(Default, Bundle)]
pub struct PlayerBundle {
    pub player: Player,

    #[bundle]
    pub sprite_bundle: SpriteBundle,

    #[bundle]
    pub physics_bundle: PhysicsBundle,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash, Reflect)]
pub struct Player;
