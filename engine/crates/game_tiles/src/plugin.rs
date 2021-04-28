use crate::{
    render::RegionData, systems::WorldRedrawEvent, RegionWorldPosition, RegionWorldRect, Tile,
    TileRegionPosition, TileRegionRect, TileWorldPosition, TileWorldRect,
};
use game_camera::CameraPlugin;
use game_core::{loading::MainLoadingMode, modes::ModeExt, GameStage, GlobalMode, ModeEvent};
use game_lib::bevy::{ecs as bevy_ecs, prelude::*};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, SystemLabel)]
pub struct TilePlugin;

impl Plugin for TilePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.register_type::<Tile>()
            // Positions
            .register_type::<TileRegionPosition>()
            .register_type::<TileWorldPosition>()
            .register_type::<RegionWorldPosition>()
            // Rects
            .register_type::<TileRegionRect>()
            .register_type::<TileWorldRect>()
            .register_type::<RegionWorldRect>()
            // Events/components
            .register_type::<WorldRedrawEvent>()
            // .register_type::<Region>()
            // .register_type::<GameWorld>()
            .add_asset::<RegionData>()
            .add_event::<WorldRedrawEvent>()
            .add_system_set_to_stage(
                GameStage::GameUpdate,
                SystemSet::new()
                    .label(TilePlugin)
                    .label(TileSystem::SetupRendering)
                    .in_ambiguity_set(MainLoadingMode::RequestAssets)
                    .with_run_criteria(Some(MainLoadingMode::RequestAssets).on(ModeEvent::Enter))
                    .with_system(crate::systems::setup_rendering.system()),
            )
            .add_system_set_to_stage(
                GameStage::GameUpdate,
                SystemSet::new()
                    .label(TilePlugin)
                    .label(TileSystem::SetupWorld)
                    .in_ambiguity_set(TileSystem::SetupWorld)
                    .with_run_criteria(GlobalMode::InGame.on(ModeEvent::Enter))
                    .with_system(crate::systems::create_game_world.system()),
            )
            .add_system_set_to_stage(
                GameStage::GameUpdate,
                SystemSet::new()
                    .label(TilePlugin)
                    .label(TileSystem::DetectRedraw)
                    .after(CameraPlugin)
                    .after(TileSystem::SetupWorld)
                    .in_ambiguity_set(TileSystem::DetectRedraw)
                    .with_run_criteria(GlobalMode::InGame.on(ModeEvent::Active))
                    .with_system(crate::systems::world_changed.system())
                    .with_system(crate::systems::camera_changed.system()),
            )
            .add_system_set_to_stage(
                GameStage::GameUpdate,
                SystemSet::new()
                    .label(TilePlugin)
                    .label(TileSystem::Redraw)
                    .after(TileSystem::DetectRedraw)
                    .with_run_criteria(GlobalMode::InGame.on(ModeEvent::Active))
                    .with_system(crate::systems::update_visible_regions.system()),
            );
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, SystemLabel, AmbiguitySetLabel)]
pub enum TileSystem {
    SetupRendering,
    SetupWorld,
    DetectRedraw,
    Redraw,
}
