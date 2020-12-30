use bevy::prelude::*;
use std::time::Duration;
use tracing::instrument;

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

pub struct TimedPlugin;

impl TimedPlugin {
    #[instrument(skip(commands, time, query))]
    fn update_lifetimes(
        commands: &mut Commands,
        time: Res<Time>,
        mut query: Query<(Entity, &mut Timed)>,
    ) {
        for (entity, mut timed) in query.iter_mut() {
            timed.update(time.delta());
            if timed.remaining() <= Duration::ZERO {
                commands.despawn_recursive(entity);
            }
        }
    }
}

impl Plugin for TimedPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_to_stage(stage::PRE_UPDATE, Self::update_lifetimes.system());
    }
}
