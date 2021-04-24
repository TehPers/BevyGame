use crate::{Drag, Gravity};
use game_lib::bevy::prelude::*;
use std::time::Duration;

#[derive(Clone, Debug, Reflect)]
pub struct PhysicsState {
    pub step_timer: Timer,
    pub queued_steps: u32,

    /// Global gravitational acceleration in `m/s^2`. This can be overriden
    /// with the `Gravity` component.
    pub gravity: Gravity,

    /// Global drag coefficient. Drag is not calculated using the shape of the
    /// body, so this will always be multiplied by the square velocity.
    pub drag: Drag,
}

impl Default for PhysicsState {
    fn default() -> Self {
        PhysicsState {
            // step_timer: Timer::new(Duration::from_secs_f32(1.0), true),
            step_timer: Timer::new(Duration::from_secs_f32(1.0 / 30.0), true),
            queued_steps: Default::default(),
            gravity: Default::default(),
            drag: Default::default(),
        }
    }
}
