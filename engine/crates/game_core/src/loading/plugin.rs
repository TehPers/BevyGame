use crate::{
    combinators::if_all, loading::systems, modes::ModeExt, AppBuilderExt, GameStage, GlobalMode,
    ModeEvent,
};
use game_lib::bevy::{ecs as bevy_ecs, prelude::*};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, SystemLabel)]
pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_mode(Option::<MainLoadingMode>::None)
            .add_system_set_to_stage(
                GameStage::GameUpdate,
                SystemSet::new()
                    .label(LoadingPlugin)
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
                GameStage::GamePreUpdate,
                SystemSet::new()
                    .label(LoadingPlugin)
                    .label(MainLoadingSystem::Setup)
                    .with_run_criteria(Some(MainLoadingMode::RequestAssets).on(ModeEvent::Enter))
                    .with_system(systems::setup_main_loading.system()),
            )
            .add_system_to_stage(
                GameStage::GamePostUpdate,
                GlobalMode::InGame
                    .transition_system()
                    .label(LoadingPlugin)
                    .label(MainLoadingSystem::CheckIfLoaded)
                    .with_run_criteria(if_all(vec![
                        Some(MainLoadingMode::WaitForAssets).on(ModeEvent::Active),
                        Box::new(systems::if_required_assets_loaded.system()),
                    ])),
            )
            .add_system_set_to_stage(
                GameStage::GamePostUpdate,
                SystemSet::new()
                    .label(LoadingPlugin)
                    .with_run_criteria(GlobalMode::MainLoading.on(ModeEvent::Exit))
                    .with_system(systems::cleanup_main_loading.system()),
            );
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, AmbiguitySetLabel)]
pub enum MainLoadingMode {
    /// While requesting assets. This state immediately transitions to
    /// [`MainLoadingMode::WaitForAssets`] after a single update tick
    RequestAssets,

    /// While waiting for required assets to load. Once all required assets are
    /// loaded, the global state is transitioned.
    WaitForAssets,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, SystemLabel)]
pub enum MainLoadingSystem {
    Setup,
    CheckIfLoaded,
}
