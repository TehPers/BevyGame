use bevy::{prelude::*, reflect::TypeUuid, render::renderer::RenderResources};

#[derive(Debug, Default, RenderResources, TypeUuid)]
#[uuid = "0a48381b-ba10-420a-ac06-fda72a6fc0d0"]
pub struct TileWorldMaterial {
    pub texture: Handle<Texture>,
}

#[derive(Debug, Default, RenderResources, TypeUuid)]
#[uuid = "b3d1e69e-2f2c-463c-8556-93d42869bab4"]
pub struct TileWorldVertexData {
    #[render_resources(buffer)]
    pub positions: Vec<Vec2>,
}
