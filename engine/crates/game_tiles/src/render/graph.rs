use crate::render::{
    node::REGION_DATA,
    pipeline::{
        build_region_pipeline, REGION_MESH_HANDLE, REGION_PIPELINE_HANDLE,
        REGION_TEXTURE_ATLAS_HANDLE,
    },
    RegionData, RegionMesh,
};
use game_core::loading::RequiredAssetLoader;
use game_lib::bevy::{
    prelude::*,
    render::{
        pipeline::PipelineDescriptor,
        render_graph::{base::node, RenderGraph, RenderResourcesNode},
    },
};

pub fn add_region_render_graph(
    graph: &mut RenderGraph,
    meshes: &mut Assets<Mesh>,
    pipelines: &mut Assets<PipelineDescriptor>,
    texture_atlases: &mut Assets<TextureAtlas>,
    asset_loader: &mut RequiredAssetLoader,
) {
    graph.add_system_node(REGION_DATA, RenderResourcesNode::<RegionData>::new(false));
    graph.add_node_edge(REGION_DATA, node::MAIN_PASS).unwrap();

    meshes.set_untracked(REGION_MESH_HANDLE, RegionMesh::default().into());
    pipelines.set_untracked(REGION_PIPELINE_HANDLE, build_region_pipeline(asset_loader));
    texture_atlases.set_untracked(
        REGION_TEXTURE_ATLAS_HANDLE,
        TextureAtlas::from_grid(
            asset_loader.load_required("tilesheets/tiles.png"),
            Vec2::new(16.0, 16.0),
            16,
            16,
        ),
    );
}
