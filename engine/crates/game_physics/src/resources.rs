use crate::{broad_phase::Entry, Drag, Gravity};
use bevy::{prelude::*, utils::HashMap};
use std::time::Duration;

#[derive(Clone, Debug, Reflect)]
pub struct PhysicsState {
    #[reflect(ignore)]
    pub interval: Duration,
    #[reflect(ignore)]
    pub lag: Duration,
    pub entry_map: HashMap<Entity, Entry>,

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
            interval: Duration::from_secs_f32(1.0 / 60.0),
            lag: Duration::default(),
            entry_map: HashMap::default(),
            gravity: Gravity::default(),
            drag: Drag::default(),
        }
    }
}

impl PhysicsState {
    /// Linear interpolation factor calculated from progress to
    /// next physics update step.
    pub fn lerp(&self) -> f32 {
        self.lag.as_secs_f32() / self.interval.as_secs_f32()
    }
}
