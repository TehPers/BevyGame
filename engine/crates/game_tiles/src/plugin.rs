use crate::{
    render::{TileWorldMaterial, TileWorldVertexData},
    systems::WorldRedrawEvent,
    Tile, TilePosition, TileRegion, TileWorld,
};
use bevy::{app::startup_stage, prelude::*};

pub struct TilePlugin;

impl Plugin for TilePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.register_type::<Tile>()
            .register_type::<TilePosition>()
            .register_type::<TileRegion>()
            .register_type::<WorldRedrawEvent>()
            .register_type::<TileWorld>()
            .add_asset::<TileWorldMaterial>()
            .add_asset::<TileWorldVertexData>()
            .add_event::<WorldRedrawEvent>()
            .add_startup_system_to_stage(
                startup_stage::PRE_STARTUP,
                crate::systems::setup_rendering.system(),
            )
            .add_startup_system_to_stage(
                startup_stage::PRE_STARTUP,
                crate::systems::create_world.system(),
            )
            .add_system_to_stage(stage::EVENT, crate::systems::update_events.system())
            .add_system_to_stage(stage::POST_UPDATE, crate::systems::world_changed.system())
            .add_system_to_stage(stage::POST_UPDATE, crate::systems::camera_changed.system())
            // TODO: this should probably be moved to a render node system
            .add_system_to_stage(stage::POST_UPDATE, crate::systems::redraw_world.system());
    }
}
