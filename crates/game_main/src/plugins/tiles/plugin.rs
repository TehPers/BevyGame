use crate::plugins::{
    random::Random,
    tiles::{GridCoordinate, Tile, TileWorld},
};
use bevy::{
    app::startup_stage,
    prelude::*,
    reflect::TypeUuid,
    render::{
        pipeline::{PipelineDescriptor, RenderPipeline},
        render_graph::{base::node, AssetRenderResourcesNode, RenderGraph},
        renderer::RenderResources,
        shader::ShaderStages,
    },
    utils::HashMap,
};
use rand::{distributions::Uniform, prelude::Distribution};
use tracing::instrument;

pub struct TilesPlugin;

impl TilesPlugin {
    const RENDER_GRAPH_NODE: &'static str = "tiles";

    #[instrument(skip(
        commands,
        asset_server,
        pipelines,
        meshes,
        materials,
        render_graph,
        random
    ))]
    fn setup(
        commands: &mut Commands,
        asset_server: Res<AssetServer>,
        mut pipelines: ResMut<Assets<PipelineDescriptor>>,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<TileWorldMaterial>>,
        mut render_graph: ResMut<RenderGraph>,
        mut random: ResMut<Random>,
    ) {
        asset_server.watch_for_changes().unwrap();

        // Create the tile rendering pipeline
        let pipeline_handle = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
            vertex: asset_server.load("shaders/test.vert"),
            fragment: Some(asset_server.load("shaders/test.frag")),
        }));

        // Bind the tile material to the shader
        render_graph.add_system_node(
            Self::RENDER_GRAPH_NODE,
            AssetRenderResourcesNode::<TileWorldMaterial>::new(true),
        );

        // Ensure that the tile material runs before the main pass
        render_graph
            .add_node_edge(Self::RENDER_GRAPH_NODE, node::MAIN_PASS)
            .unwrap();

        let tile_world_material = materials.add(TileWorldMaterial {
            color: Color::WHITE,
            tile_sheet: asset_server.load("tilesheets/tiles.png"),
        });

        // Generate world
        const WORLD_WIDTH: u32 = 8;
        const WORLD_HEIGHT: u32 = 8;
        let mut world = TileWorld::new(WORLD_WIDTH, WORLD_HEIGHT).unwrap();
        let dist = Uniform::new_inclusive(0, 2);
        for x in 0..WORLD_WIDTH {
            for y in 0..WORLD_HEIGHT {
                *world.get_mut(x, y).unwrap() = match dist.sample(&mut *random) {
                    1 => Some(Tile::Stone),
                    2 => Some(Tile::Dirt),
                    _ => None,
                }
            }
        }

        let tile_mesh = meshes.add(Mesh::from(&world));
        commands
            .spawn(MeshBundle {
                mesh: tile_mesh,
                render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                    pipeline_handle,
                )]),
                ..Default::default()
            })
            .with(tile_world_material)
            .with(world);
    }

    #[instrument(skip(meshes, query))]
    fn update_world_mesh(
        mut meshes: ResMut<Assets<Mesh>>,
        query: Query<(&TileWorld, &Handle<Mesh>), Changed<TileWorld>>,
    ) {
        for (world, mesh) in query.iter() {
            if let Some(mesh) = meshes.get_mut(mesh) {
                *mesh = world.into();
            }
        }
    }

    #[instrument(skip(commands, world_query, tile_query))]
    fn update_tile_markers(
        commands: &mut Commands,
        world_query: Query<&TileWorld, Changed<TileWorld>>,
        mut tile_query: Query<(&mut TileMarker, Entity)>,
    ) {
        for world in world_query.iter() {
            let mut existing_tiles: HashMap<_, _> = tile_query
                .iter_mut()
                .map(|(marker, entity)| ((marker.coords.0, marker.coords.1), (marker, entity)))
                .collect();

            // Iterate over all tiles
            for (x, y, tile) in world.iter_tiles() {
                // Check if the tile already has an entity associated with it
                match existing_tiles.remove(&(x, y)) {
                    // Tile was changed
                    Some((mut marker, _)) if marker.tile != tile => {
                        marker.tile = tile;
                    }

                    // Tile was added
                    None => {
                        Self::create_entity_for_tile(commands, x, y, tile);
                    }

                    // Tile is unchanged
                    _ => {}
                }
            }

            // Despawn removed tiles
            for (_, (_, entity)) in existing_tiles {
                commands.despawn(entity);
            }
        }
    }

    fn create_entity_for_tile(
        commands: &mut Commands,
        x: GridCoordinate,
        y: GridCoordinate,
        tile: Tile,
    ) -> Entity {
        commands
            .spawn((TileMarker {
                coords: (x, y),
                tile,
            },))
            .current_entity()
            .unwrap()
    }
}

impl Plugin for TilesPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_asset::<TileWorldMaterial>()
            // .add_startup_system_to_stage(startup_stage::STARTUP, Self::setup.system())
            .add_system_to_stage(stage::POST_UPDATE, Self::update_tile_markers.system())
            .add_system_to_stage(stage::UPDATE, Self::update_world_mesh.system());
    }
}

#[derive(RenderResources, Default, TypeUuid)]
#[uuid = "7761fb93-636a-4ebd-ab89-8b9762f5cf64"]
pub struct TileWorldMaterial {
    pub color: Color,
    pub tile_sheet: Handle<Texture>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct TileMarker {
    coords: (GridCoordinate, GridCoordinate),
    tile: Tile,
}
