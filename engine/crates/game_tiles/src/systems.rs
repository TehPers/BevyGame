use crate::{
    generation::TerrainWorldGenerator,
    render::{add_region_render_graph, RegionBundle, RegionData},
    GameWorld, GameWorldGetError, RegionWorldPosition, RegionWorldRect, TileWorldPosition,
    TileWorldRect,
};
use game_camera::{ProjectionExt, ScaledOrthographicProjection};
use game_core::systems::RequiredAssetLoader;
use game_lib::{
    bevy::{
        prelude::*,
        render::{camera::Camera, pipeline::PipelineDescriptor, render_graph::RenderGraph},
        utils::HashMap,
    },
    tracing::{self, instrument},
};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Reflect)]
pub struct WorldRedrawEvent {
    pub world_changed: bool,
}

#[instrument(skip(asset_loader, meshes, pipelines, texture_atlases, render_graph))]
pub fn setup_rendering(
    mut asset_loader: RequiredAssetLoader,
    mut meshes: ResMut<Assets<Mesh>>,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut render_graph: ResMut<RenderGraph>,
) {
    asset_loader.asset_server.watch_for_changes().unwrap();
    add_region_render_graph(
        &mut render_graph,
        &mut meshes,
        &mut pipelines,
        &mut texture_atlases,
        &mut asset_loader,
    );
}

#[instrument(skip(commands))]
pub fn create_game_world(mut commands: Commands) {
    // let generator = FlatWorldGenerator::new(Tile::Stone, None);
    // let generator = FlatWorldGenerator::new(Tile::Stone, Some(30));
    let generator = TerrainWorldGenerator::new_random(&mut game_lib::rand::thread_rng());
    let world = GameWorld::new(Box::new(generator));
    commands.insert_resource(world);
}

#[instrument(skip(redraw_event, world))]
pub fn world_changed(mut redraw_event: EventWriter<WorldRedrawEvent>, world: Res<GameWorld>) {
    if world.is_changed() {
        redraw_event.send(WorldRedrawEvent {
            world_changed: true,
        });
    }
}

#[instrument(skip(redraw_event, query))]
pub fn camera_changed(
    mut redraw_event: EventWriter<WorldRedrawEvent>,
    query: Query<
        (),
        (
            With<Camera>,
            With<ScaledOrthographicProjection>,
            Or<(Changed<Transform>, Changed<ScaledOrthographicProjection>)>,
        ),
    >,
) {
    for _ in query.iter() {
        redraw_event.send(WorldRedrawEvent {
            world_changed: false,
        });
    }
}

#[instrument(skip(
    commands,
    last_rect,
    redraw_event,
    region_data,
    windows,
    world,
    region_query,
    camera_query
))]
pub fn update_visible_regions(
    mut commands: Commands,
    mut last_rect: Local<RegionWorldRect>,
    mut redraw_event: EventReader<WorldRedrawEvent>,
    mut region_data: ResMut<Assets<RegionData>>,
    windows: Res<Windows>,
    mut world: ResMut<GameWorld>,
    region_query: Query<(Entity, &RegionWorldPosition, &Handle<RegionData>)>,
    camera_query: Query<(&ScaledOrthographicProjection, &Camera, &Transform)>,
) {
    if let Some(world_changed) = redraw_event.iter().fold(None, |acc, cur| {
        acc.map(|changed| changed | cur.world_changed)
            .or(Some(cur.world_changed))
    }) {
        let (projection, camera, camera_transform) = match camera_query.single() {
            Ok(result) => result,
            Err(_) => return,
        };

        let window = match windows.get(camera.window) {
            Some(window) => window,
            None => return,
        };

        // Get rectangle of visible regions
        let screen_size = Vec2::new(window.width(), window.height());
        let world_visible_bottom_left: Vec2 = projection
            .screen_to_world(camera_transform, Vec2::new(0.0, 0.0), screen_size)
            .0
            .floor()
            .into();
        let world_visible_top_right: Vec2 = projection
            .screen_to_world(camera_transform, screen_size, screen_size)
            .0
            .ceil()
            .into();
        let visible_rect = TileWorldRect::new(
            world_visible_bottom_left.into(),
            (world_visible_top_right - world_visible_bottom_left).into(),
        );
        let visible_rect: RegionWorldRect = RegionWorldRect::from(visible_rect).expand(1);

        // Only update if needed
        if !world_changed && visible_rect == *last_rect {
            return;
        }
        *last_rect = visible_rect;

        let mut visible_regions: HashMap<_, _> = region_query
            .iter()
            .map(|(entity, &position, data)| (position, (entity, data)))
            .collect();

        for position in visible_rect.iter_positions() {
            // Only call `get_or_generate_region` if needed because it always
            // flags `world` as changed regardless of if the region has already
            // been generated
            // TODO: Track if the world actually has been updated
            let region = match world.get_region(position) {
                Ok(region) => region,
                Err(GameWorldGetError::NotYetGenerated) => world.get_or_generate_region(position),
            };

            // Remove the entity to prevent it from being despawned later
            match visible_regions.remove(&position) {
                Some((_, region_data_handle)) => {
                    // Update existing region entity
                    if let Some(region_data) = region_data.get_mut(region_data_handle) {
                        *region_data = RegionData::from(region);
                    }
                }
                None => {
                    // Create new region entity
                    let region_world_pos = TileWorldPosition::from(position);
                    commands.spawn_bundle(RegionBundle {
                        position,
                        region_data: region_data.add(RegionData::from(region)),
                        transform: Transform::from_xyz(
                            region_world_pos.x as f32,
                            region_world_pos.y as f32,
                            0.0,
                        ),
                        ..Default::default()
                    });
                }
            }
        }

        // Despawn invisible region entities
        for (_, (entity, _)) in visible_regions {
            commands.entity(entity).despawn();
        }
    }
}
