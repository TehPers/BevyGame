use crate::{Region, TileRegionPosition};
use game_lib::bevy::{
    prelude::*,
    render::{mesh::Indices, pipeline::PrimitiveTopology},
};

#[derive(Clone, Debug)]
pub struct RegionMesh {
    pub vertices: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub tile_indices: Vec<i32>,
    pub indices: Vec<u32>,
}

impl Default for RegionMesh {
    fn default() -> Self {
        // TODO: Remove duplicate vertices
        let mut vertices = Vec::with_capacity(Region::TILES * 4);
        let mut uvs = Vec::with_capacity(Region::TILES * 4);
        let mut tile_indices = Vec::with_capacity(Region::TILES * 4);
        let mut indices = Vec::with_capacity(Region::TILES * 6);
        let mut index = 0;
        let mut tile_index = 0;
        for TileRegionPosition { x, y } in Region::BOUNDS.iter_positions() {
            let x = x as f32;
            let y = y as f32;

            // Northwest
            vertices.push([x, y + 1.0, 0.0]);
            uvs.push([0.0, 0.0]);
            tile_indices.push(tile_index);
            // Southwest
            vertices.push([x, y, 0.0]);
            uvs.push([0.0, 1.0]);
            tile_indices.push(tile_index);
            // Southeast
            vertices.push([x + 1.0, y, 0.0]);
            uvs.push([1.0, 1.0]);
            tile_indices.push(tile_index);
            // Northeast
            vertices.push([x + 1.0, y + 1.0, 0.0]);
            uvs.push([1.0, 0.0]);
            tile_indices.push(tile_index);

            // Northwest corner
            indices.push(index + 1);
            indices.push(index + 3);
            indices.push(index);
            // Southeast corner
            indices.push(index + 1);
            indices.push(index + 2);
            indices.push(index + 3);

            index += 4;
            tile_index += 1;
        }

        RegionMesh {
            vertices,
            uvs,
            tile_indices,
            indices,
        }
    }
}

impl From<RegionMesh> for Mesh {
    fn from(value: RegionMesh) -> Self {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, value.vertices);
        mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, value.uvs);
        mesh.set_attribute("Vertex_Tile_Index", value.tile_indices);
        mesh.set_indices(Some(Indices::U32(value.indices)));
        mesh
    }
}
