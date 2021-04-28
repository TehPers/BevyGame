use crate::{
    render::{
        pipeline::{REGION_MESH_HANDLE, REGION_PIPELINE_HANDLE, REGION_TEXTURE_ATLAS_HANDLE},
        RegionData,
    },
    RegionWorldPosition,
};
use game_lib::bevy::{
    ecs as bevy_ecs,
    prelude::*,
    render::{pipeline::RenderPipeline, render_graph::base::MainPass},
};

#[derive(Clone, Debug, Bundle)]
pub struct RegionBundle {
    pub position: RegionWorldPosition,
    pub mesh: Handle<Mesh>,
    pub region_data: RegionData,
    pub texture_atlas: Handle<TextureAtlas>,
    pub main_pass: MainPass,
    pub draw: Draw,
    pub visible: Visible,
    pub render_pipelines: RenderPipelines,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl RegionBundle {
    pub fn new_defaults(region_data: RegionData) -> Self {
        RegionBundle {
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                REGION_PIPELINE_HANDLE.typed(),
            )]),
            mesh: REGION_MESH_HANDLE.typed(),
            texture_atlas: REGION_TEXTURE_ATLAS_HANDLE.typed(),
            region_data,
            position: Default::default(),
            main_pass: Default::default(),
            draw: Default::default(),
            visible: Default::default(),
            transform: Default::default(),
            global_transform: Default::default(),
        }
    }
}
