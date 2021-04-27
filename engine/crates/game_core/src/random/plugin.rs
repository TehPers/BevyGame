use crate::{
    random::{RandomConfig, ResetRandom},
    GameStage,
};
use game_lib::{
    bevy::{ecs as bevy_ecs, prelude::*},
    rand_pcg::Pcg64Mcg,
};

pub type GameRandom = Pcg64Mcg;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, SystemLabel)]
pub struct RandomPlugin;

impl Plugin for RandomPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_event::<ResetRandom>()
            .add_system_set_to_stage(
                GameStage::Startup,
                SystemSet::new()
                    .label(RandomPlugin)
                    .with_system(crate::random::systems::setup.system()),
            )
            .add_system_set_to_stage(
                GameStage::GamePreUpdate,
                SystemSet::new()
                    .label(RandomPlugin)
                    .with_system(crate::random::systems::reset_random.system()),
            );

        let world = app.world_mut();
        if !world.contains_resource::<RandomConfig>() {
            world.insert_resource(RandomConfig::default());
        }
    }
}
