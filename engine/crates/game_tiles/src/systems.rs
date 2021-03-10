use crate::{
    render::{TileWorldBundle, TileWorldGraphBuilder, TileWorldMaterial, TileWorldVertexData},
    Tile, TileCoordinate, TilePosition, TileRegion, TileWorld, TILE_SHEET_COL_LENGTH,
    TILE_SHEET_ROW_LENGTH, TILE_UV_SIZE,
};
use bevy::{
    prelude::*,
    render::{
        camera::Camera,
        mesh::Indices,
        pipeline::{PipelineDescriptor, PrimitiveTopology},
        render_graph::RenderGraph,
    },
};
use game_camera::{ProjectionExt, ScaledOrthographicProjection};
use std::convert::TryInto;
use tracing::instrument;

const WORLD_WIDTH: TileCoordinate = 1024;
const WORLD_HEIGHT: TileCoordinate = 1024;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Reflect)]
pub struct WorldRedrawEvent {
    force_redraw: bool,
}

#[instrument(skip(asset_server, pipelines, render_graph))]
pub fn setup_rendering(
    asset_server: Res<AssetServer>,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut render_graph: ResMut<RenderGraph>,
) {
    asset_server.watch_for_changes().unwrap();
    render_graph.add_tile_world_graph(&mut *pipelines, &*asset_server);
}

#[instrument(skip(commands, asset_server, materials, vertex_data_assets, meshes))]
pub fn create_world(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<TileWorldMaterial>>,
    mut vertex_data_assets: ResMut<Assets<TileWorldVertexData>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // Generate the world
    let world = info_span!("world_gen").in_scope(|| {
        let mut world = TileWorld::new(WORLD_WIDTH, WORLD_HEIGHT).unwrap();
        for x in 0..world.size().x {
            for y in 0..world.size().y {
                *world.get_mut(TilePosition::new(x, y)).unwrap() = Some(Tile::Stone);
            }
        }

        world
    });

    // Spawn the world
    info!("spawning world");
    let (mesh, vertex_data) = get_world_render_data(&world, TileRegion::default());
    commands.insert_resource(world).spawn(TileWorldBundle {
        material: materials.add(TileWorldMaterial {
            texture: asset_server.load("tilesheets/tiles.png"),
        }),
        mesh: meshes.add(mesh),
        vertex_data: vertex_data_assets.add(vertex_data),
        ..Default::default()
    });
}

#[instrument(skip(redraw_event))]
pub fn update_events(mut redraw_event: ResMut<Events<WorldRedrawEvent>>) {
    redraw_event.update();
}

#[instrument(skip(redraw_event, _world))]
pub fn world_changed(
    mut redraw_event: ResMut<Events<WorldRedrawEvent>>,
    _world: ChangedRes<TileWorld>,
) {
    redraw_event.send(WorldRedrawEvent {
        force_redraw: false,
    });
}

#[instrument(skip(redraw_event, query))]
pub fn camera_changed(
    mut redraw_event: ResMut<Events<WorldRedrawEvent>>,
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
        redraw_event.send(WorldRedrawEvent { force_redraw: true });
    }
}

#[instrument(skip(
    redraw_event,
    redraw_event_reader,
    meshes,
    vertex_data_assets,
    windows,
    world,
    world_entity_query,
    camera_query
))]
pub fn redraw_world(
    mut drawn_region: Local<Option<TileRegion>>,
    redraw_event: Res<Events<WorldRedrawEvent>>,
    mut redraw_event_reader: Local<EventReader<WorldRedrawEvent>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut vertex_data_assets: ResMut<Assets<TileWorldVertexData>>,
    windows: Res<Windows>,
    world: Res<TileWorld>,
    mut world_entity_query: Query<(&mut Handle<Mesh>, &mut Handle<TileWorldVertexData>)>,
    camera_query: Query<(&ScaledOrthographicProjection, &Camera, &Transform)>,
) {
    for event in redraw_event_reader.iter(&redraw_event) {
        let success = (|| {
            let (mut mesh_handle, mut vertex_data_handle) = world_entity_query.iter_mut().nth(0)?;
            let (projection, camera, transform) = camera_query.iter().nth(0)?;
            let window = windows.get(camera.window)?;
            let screen_size = Vec2::new(window.width(), window.height());
            let region_bottom_left: Vec2 = projection
                .screen_to_world(transform, Vec2::new(0.0, 0.0), screen_size)
                .0
                .floor()
                .into();
            let region_top_right: Vec2 = projection
                .screen_to_world(transform, screen_size, screen_size)
                .0
                .ceil()
                .into();
            let region = TileRegion::new(
                region_bottom_left.into(),
                (region_top_right - region_bottom_left).into(),
            );

            if !event.force_redraw && drawn_region.filter(|&r| r == region).is_some() {
                return Some(());
            }

            let (new_mesh, new_vertex_data) = get_world_render_data(&*world, region);

            // Update mesh
            if let Some(mesh) = meshes.get_mut(&*mesh_handle) {
                *mesh = new_mesh;
            } else {
                warn!("missing mesh for world, recreating it");
                *mesh_handle = meshes.add(new_mesh);
            }

            // Update vertices
            if let Some(vertex_data) = vertex_data_assets.get_mut(&*vertex_data_handle) {
                *vertex_data = new_vertex_data;
            } else {
                warn!("missing vertex data for world, recreating it");
                *vertex_data_handle = vertex_data_assets.add(new_vertex_data);
            }

            *drawn_region = Some(region);
            Some(())
        })()
        .is_some();

        if !success {
            warn!("failure updating world render info");
        }
    }
}

#[instrument(skip(world))]
fn get_world_render_data(world: &TileWorld, region: TileRegion) -> (Mesh, TileWorldVertexData) {
    let size = region.size();
    let tiles = world
        .iter_rect(region)
        .filter_map(|(pos, tile)| tile.ok().flatten().map(move |tile| (pos, tile)));

    // Get vertices
    let tile_count: usize = (size.x * size.y).try_into().unwrap();
    let vertex_count: usize = ((size.x + 1) * (size.y + 1)).try_into().unwrap();
    let /* mut */ positions: Vec<Vec2> = Vec::with_capacity(vertex_count);
    let mut expected_positions: Vec<[f32; 3]> = Vec::with_capacity(tile_count * 4);
    // let mut position_indices: Vec<u32> = Vec::with_capacity(tile_count * 4);
    let mut indices: Vec<u32> = Vec::with_capacity(tile_count * 6);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(tile_count * 4);
    let mut colors: Vec<[f32; 4]> = Vec::with_capacity(tile_count * 4);
    // let mut index_map: HashMap<[u32; 2], u32> = HashMap::with_capacity(vertex_count);
    // let mut get_position_index = |vertex: [u32; 2]| {
    //     *index_map.entry(vertex).or_insert_with(|| {
    //         let index = positions.len();
    //         let [x, y] = vertex;
    //         positions.push(Vec2::new(x as f32 - 0.5, y as f32 - 0.5));
    //         index.try_into().unwrap()
    //     })
    // };
    for (pos, tile) in tiles {
        let (u, v) = tile
            .index()
            .into_uv(TILE_SHEET_ROW_LENGTH, TILE_SHEET_COL_LENGTH);

        let vertices = [
            // Northwest
            ([pos.x, pos.y + 1], [u, v], Color::WHITE.into()),
            // Southwest
            ([pos.x, pos.y], [u, v + TILE_UV_SIZE], Color::WHITE.into()),
            // Southeast
            (
                [pos.x + 1, pos.y],
                [u + TILE_UV_SIZE, v + TILE_UV_SIZE],
                Color::WHITE.into(),
            ),
            // Northeast
            (
                [pos.x + 1, pos.y + 1],
                [u + TILE_UV_SIZE, v],
                Color::WHITE.into(),
            ),
        ];

        // Vertex buffer data
        let base_index: u32 = uvs.len().try_into().unwrap();
        for &([x, y], uv, color) in vertices.iter() {
            // position_indices.push(get_position_index([x, y]));
            expected_positions.push([x as f32, y as f32, 0.0]);
            uvs.push(uv);
            colors.push(color);
        }

        // Indices
        for &index in [1, 3, 0, 1, 2, 3].iter() {
            indices.push(base_index + index);
        }
    }

    // Create mesh
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(Indices::U32(indices)));
    // mesh.set_attribute("Vertex_Position_Index", position_indices);
    mesh.set_attribute("Vertex_Expected_Position", expected_positions);
    mesh.set_attribute("Vertex_Uv", uvs);
    mesh.set_attribute("Vertex_Color", colors);

    (mesh, TileWorldVertexData { positions })
}
