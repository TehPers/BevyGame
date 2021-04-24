use crate::render::{
    node::{REGION_DATA, TEST_PASS, TEST_PASS_CAMERA, TEST_PASS_TEXTURE},
    pipeline::{
        build_region_pipeline, REGION_MESH_HANDLE, REGION_PIPELINE_HANDLE,
        REGION_TEXTURE_ATLAS_HANDLE,
    },
    RegionData, RegionMesh, TestNode
};
use game_core::systems::RequiredAssetLoader;
use game_lib::bevy::{
    prelude::*,
    render::{
        pass::{
            LoadOp, Operations, PassDescriptor, RenderPassColorAttachmentDescriptor,
            RenderPassDepthStencilAttachmentDescriptor, TextureAttachment,
        },
        pipeline::PipelineDescriptor,
        render_graph::{base::node, AssetRenderResourcesNode, CameraNode, PassNode, RenderGraph},
    },
};

// #[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
// pub struct TestPass;

// pub fn add_test_render_graph(graph: &mut RenderGraph) {
//     let mut pass_node = PassNode::<&TestPass>::new(PassDescriptor {
//         color_attachments: vec![RenderPassColorAttachmentDescriptor {
//             attachment: TextureAttachment::Input("color_attachment".to_string()),
//             resolve_target: None,
//             ops: Operations {
//                 load: LoadOp::Clear(Color::rgb(0.1, 0.2, 0.3)),
//                 store: true,
//             },
//         }],
//         depth_stencil_attachment: Some(RenderPassDepthStencilAttachmentDescriptor {
//             attachment: TextureAttachment::Input("depth".to_string()),
//             depth_ops: Some(Operations {
//                 load: LoadOp::Clear(1.0),
//                 store: true,
//             }),
//             stencil_ops: None,
//         }),
//         sample_count: 1,
//     });
//     pass_node.add_camera(TEST_PASS_CAMERA);

//     graph.add_node(TEST_PASS, pass_node);
//     graph.add_system_node(TEST_PASS_CAMERA, CameraNode::new(TEST_PASS_CAMERA));
//     graph.add_node_edge(TEST_PASS_CAMERA, TEST_PASS).unwrap();

//     // Texture node
//     graph.add_node(TEST_PASS_TEXTURE, TestNode::new());
// }

pub fn add_region_render_graph(
    graph: &mut RenderGraph,
    meshes: &mut Assets<Mesh>,
    pipelines: &mut Assets<PipelineDescriptor>,
    texture_atlases: &mut Assets<TextureAtlas>,
    asset_loader: &mut RequiredAssetLoader,
) {
    graph.add_system_node(
        REGION_DATA,
        AssetRenderResourcesNode::<RegionData>::new(false),
    );
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
