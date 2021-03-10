use crate::render::{pipeline::PIPELINE_HANDLE, TileWorldMaterial};
use bevy::{
    prelude::*,
    render::{pipeline::RenderPipeline, render_graph::base::MainPass},
};

use super::TileWorldVertexData;

#[derive(Bundle)]
pub struct TileWorldBundle {
    pub material: Handle<TileWorldMaterial>,
    pub mesh: Handle<Mesh>,
    pub vertex_data: Handle<TileWorldVertexData>,
    pub main_pass: MainPass,
    pub draw: Draw,
    pub visible: Visible,
    pub render_pipelines: RenderPipelines,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl Default for TileWorldBundle {
    fn default() -> Self {
        TileWorldBundle {
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                PIPELINE_HANDLE.typed(),
            )]),
            mesh: Default::default(),
            material: Default::default(),
            vertex_data: Default::default(),
            main_pass: Default::default(),
            draw: Default::default(),
            visible: Default::default(),
            transform: Default::default(),
            global_transform: Default::default(),
        }
    }
}
