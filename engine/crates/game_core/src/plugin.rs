use crate::{combinators::if_all, systems, AppBuilderExt, ModeEvent, modes::ModeExt};
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
            GameStage::PreUpdate,
            SystemStage::parallel(),
        )
        .add_stage_after(
            CoreStage::Update,
            GameStage::Update,
            SystemStage::parallel(),
        )
        .add_stage_after(
            CoreStage::PostUpdate,
            GameStage::PostUpdate,
            SystemStage::parallel(),
        )
        // Modes
        .add_stage_before(
            CoreStage::PreUpdate,
            ModeStage::UpdateModes,
            SystemStage::parallel(),
        )
        .add_mode(GlobalMode::MainLoading)
        .add_mode(Option::<MainLoadingMode>::None)
        // Main loading
        .add_system_set_to_stage(
            GameStage::Update,
            SystemSet::new()
                .label(CorePlugin)
                .in_ambiguity_set(GlobalMode::MainLoading)
                .with_system(
                    // Request assets after main loading begins
                    Some(MainLoadingMode::RequestAssets)
                        .transition_system()
                        .with_run_criteria(GlobalMode::MainLoading.on(ModeEvent::Enter)),
                )
                .with_system(
                    // Wait for assets after requesting assets
                    Some(MainLoadingMode::WaitForAssets)
                        .transition_system()
                        .with_run_criteria(
                            Some(MainLoadingMode::RequestAssets).on(ModeEvent::Enter),
                        ),
                )
                .with_system(
                    // Disable loading mode after main loading finishes
                    Option::<MainLoadingMode>::None
                        .transition_system()
                        .with_run_criteria(GlobalMode::MainLoading.on(ModeEvent::Exit)),
                ),
        )
        .add_system_set_to_stage(
            GameStage::PreUpdate,
            SystemSet::new()
                .label(CorePlugin)
                .label(MainLoadingSystem::Setup)
                .with_run_criteria(Some(MainLoadingMode::RequestAssets).on(ModeEvent::Enter))
                .with_system(systems::setup_main_loading.system()),
        )
        .add_system_to_stage(
            GameStage::PostUpdate,
            GlobalMode::InGame
                .transition_system()
                .label(CorePlugin)
                .label(MainLoadingSystem::CheckIfLoaded)
                .with_run_criteria(if_all(vec![
                    Some(MainLoadingMode::WaitForAssets).on(ModeEvent::Active),
                    Box::new(systems::if_required_assets_loaded.system()),
                ])),
        )
        .add_system_set_to_stage(
            GameStage::PostUpdate,
            SystemSet::new()
                .label(CorePlugin)
                .with_run_criteria(GlobalMode::MainLoading.on(ModeEvent::Exit))
                .with_system(systems::cleanup_main_loading.system()),
        );
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, StageLabel)]
pub enum GameStage {
    /// Ran once when the game first opens, after [`CoreStage::Startup`].
    /// Prefer to use `RunOnce` and states/events instead if possible.
    Startup,

    /// Ran after [`CoreStage::PreUpdate`].
    PreUpdate,

    /// Ran after [`CoreStage::Update`].
    Update,

    /// Ran after [`CoreStage::PostUpdate`].
    PostUpdate,
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

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, AmbiguitySetLabel)]
pub enum MainLoadingMode {
    /// While requesting assets. This state immediately transitions to
    /// [`MainLoadingState::WaitForAssets`] after a single update tick
    RequestAssets,

    /// While waiting for required assets to load. Once all required assets are
    /// loaded, the global state is transitioned.
    WaitForAssets,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, SystemLabel)]
pub enum MainLoadingSystem {
    /// Stage: [`GameStage::PreUpdate`]
    Setup,

    /// Stage: [`GameStage::PostUpdate`]
    CheckIfLoaded,
}
