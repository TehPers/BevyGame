use crate::AppBuilderExt;
use game_lib::bevy::{
    ecs::{self as bevy_ecs, schedule::RunOnce},
    prelude::*,
};
use std::{fmt::Debug, hash::Hash};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, SystemLabel)]
pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_stage_after(
            CoreStage::Startup,
            GameStage::Startup,
            SystemStage::parallel().with_run_criteria(RunOnce::default()),
        )
        .add_stage_after(
            CoreStage::PreUpdate,
            GameStage::GamePreUpdate,
            SystemStage::parallel(),
        )
        .add_stage_after(
            CoreStage::Update,
            GameStage::GameUpdate,
            SystemStage::parallel(),
        )
        .add_stage_after(
            CoreStage::PostUpdate,
            GameStage::GamePostUpdate,
            SystemStage::parallel(),
        )
        // Modes
        .add_stage_before(
            CoreStage::PreUpdate,
            ModeStage::UpdateModes,
            SystemStage::parallel(),
        )
        .add_mode(GlobalMode::MainLoading);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, StageLabel)]
pub enum GameStage {
    /// Ran once when the game first opens, after [`CoreStage::Startup`].
    /// Prefer to use `RunOnce` and states/events instead if possible.
    Startup,

    /// Ran after [`CoreStage::PreUpdate`].
    GamePreUpdate,

    /// Ran after [`CoreStage::Update`].
    GameUpdate,

    /// Ran after [`CoreStage::PostUpdate`].
    GamePostUpdate,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, StageLabel)]
pub enum ModeStage {
    /// Ran before [`CoreStage::PreUpdate`], and updates all the mode states.
    UpdateModes,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, AmbiguitySetLabel)]
pub enum GlobalMode {
    /// The initial game load, when the game is first started
    MainLoading,

    /// Whenever a world is being initially loaded
    WorldLoading,

    /// While playing in a world
    InGame,
}
