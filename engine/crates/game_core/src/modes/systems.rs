use std::fmt::Debug;

use crate::modes::Mode;
use game_lib::{
    bevy::{ecs::component::Component, prelude::*},
    tracing::{self, instrument},
};

#[instrument(skip(mode))]
pub fn update_mode<T: Component + Debug + Eq>(mut mode: ResMut<Mode<T>>) {
    mode.update();
}
