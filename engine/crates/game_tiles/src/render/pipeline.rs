use game_core::systems::RequiredAssetLoader;
use game_lib::bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{pipeline::PipelineDescriptor, shader::ShaderStages},
};

pub const REGION_PIPELINE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(PipelineDescriptor::TYPE_UUID, 0x5BA3E190095C409A);
pub const REGION_MESH_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Mesh::TYPE_UUID, 0x3F8EB05B6CD0403A);
pub const REGION_TEXTURE_ATLAS_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(TextureAtlas::TYPE_UUID, 0x3A06EC1D04544655);

pub fn build_region_pipeline(asset_loader: &mut RequiredAssetLoader) -> PipelineDescriptor {
    PipelineDescriptor {
        name: Some("region".into()),
        ..PipelineDescriptor::default_config(ShaderStages {
            vertex: asset_loader.load_required("shaders/region.vert"),
            fragment: Some(asset_loader.load_required("shaders/region.frag")),
        })
    }
}
