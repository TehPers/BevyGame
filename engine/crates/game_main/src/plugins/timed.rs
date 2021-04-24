use game_core::GameStage;
use game_lib::{
    bevy::{ecs as bevy_ecs, prelude::*},
    tracing::{self, instrument},
};
use std::time::Duration;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct Timed(Duration);

impl Timed {
    pub fn new(lifetime: Duration) -> Self {
        Timed(lifetime)
    }

    pub fn remaining(&self) -> Duration {
        self.0
    }

    pub fn update(&mut self, elapsed: Duration) {
        self.0 = self.0.saturating_sub(elapsed);
    }

    pub fn extend(&mut self, extension: Duration) {
        self.0 = self.0.saturating_add(extension);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, SystemLabel)]
pub struct TimedPlugin;

impl TimedPlugin {
    #[instrument(skip(commands, time, query))]
    fn update_lifetimes(
        mut commands: Commands,
        time: Res<Time>,
        mut query: Query<(Entity, &mut Timed)>,
    ) {
        for (entity, mut timed) in query.iter_mut() {
            timed.update(time.delta());
            if timed.remaining() <= Duration::ZERO {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

impl Plugin for TimedPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_set_to_stage(
            GameStage::PreUpdate,
            SystemSet::new()
                .label(TimedPlugin)
                .with_system(Self::update_lifetimes.system()),
        );
    }
}
