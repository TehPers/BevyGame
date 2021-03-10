use crate::render::{
    pipeline::{build_pipeline, PIPELINE_HANDLE},
    TileWorldMaterial,
};
use bevy::{
    prelude::*,
    render::{
        pipeline::PipelineDescriptor,
        render_graph::{base::node, AssetRenderResourcesNode, RenderGraph, RenderResourcesNode},
    },
};
use tracing::instrument;

use super::TileWorldVertexData;

pub trait TileWorldGraphBuilder {
    fn add_tile_world_graph(
        &mut self,
        pipelines: &mut Assets<PipelineDescriptor>,
        asset_server: &AssetServer,
    ) -> &mut Self;
}

impl TileWorldGraphBuilder for RenderGraph {
    #[instrument(skip(self, pipelines, asset_server))]
    fn add_tile_world_graph(
        &mut self,
        pipelines: &mut Assets<PipelineDescriptor>,
        asset_server: &AssetServer,
    ) -> &mut Self {
        // Tile world material
        self.add_system_node(
            crate::render::node::TILE_WORLD_MATERIAL,
            AssetRenderResourcesNode::<TileWorldMaterial>::new(true),
        );
        self.add_node_edge(crate::render::node::TILE_WORLD_MATERIAL, node::MAIN_PASS)
            .unwrap();

        // Tile world vertex data
        self.add_system_node(
            crate::render::node::TILE_WORLD_VERTEX_DATA,
            RenderResourcesNode::<TileWorldVertexData>::new(false),
        );

        // Pipeline
        pipelines.set_untracked(PIPELINE_HANDLE, build_pipeline(asset_server));

        self
    }
}
