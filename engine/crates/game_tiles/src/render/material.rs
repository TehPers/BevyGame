use crate::Region;
use game_lib::bevy::{
    asset as bevy_asset,
    core::{self as bevy_core, Byteable},
    prelude::*,
    reflect::TypeUuid,
    render::{
        self as bevy_render,
        renderer::{RenderResource, RenderResources},
    },
};

#[derive(Debug, RenderResources, TypeUuid)]
#[uuid = "ffa702fe-f6f0-473c-be92-c48e13eec041"]
pub struct RegionData {
    pub tile_data: Box<[RegionTileData; Region::TILES]>,
}

pub struct RegionBuffer {
    pub buffer: Handle<Texture>,
}

impl From<&Region> for RegionData {
    fn from(region: &Region) -> Self {
        let tile_data: [_; Region::TILES] =
            array_init::from_iter(Region::BOUNDS.iter_positions().map(|position| {
                let tile = region.get(position).unwrap();
                let atlas_index = tile.map(|tile| tile.index().0.into()).unwrap_or(-1);
                RegionTileData {
                    tile_color: Color::WHITE.into(),
                    atlas_index,
                    padding: Default::default(),
                }
            }))
            .unwrap();

        RegionData {
            tile_data: Box::new(tile_data),
        }
    }
}

impl From<&mut Region> for RegionData {
    fn from(region: &mut Region) -> Self {
        RegionData::from(&*region)
    }
}

#[derive(Clone, Copy, Debug, Default, RenderResource, TypeUuid)]
#[uuid = "fe1239e5-9e5e-4f1e-a485-6eedc0cb5968"]
#[repr(C)]
pub struct RegionTileData {
    pub tile_color: Vec4,
    pub atlas_index: i32,
    padding: [i32; 3],
}

unsafe impl Byteable for RegionTileData {}

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
