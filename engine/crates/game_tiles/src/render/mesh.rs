use std::array::IntoIter;

use crate::Region;
use game_lib::bevy::{
    prelude::*,
    render::{mesh::Indices, pipeline::PrimitiveTopology},
};

#[derive(Clone, Debug)]
pub struct RegionMesh {
    pub indices: Vec<u32>,
}

impl Default for RegionMesh {
    fn default() -> Self {
        let indices = (0..Region::TILES as u32)
            .map(|index| index * 4)
            .flat_map(|index| {
                // Offsets are in the format 0bXY, where:
                // - X = 0 is west, X = 1 is east
                // - Y = 0 is south, Y = 1 is north

                let tile_indices = [
                    // Northwest corner
                    index + 0b00,
                    index + 0b11,
                    index + 0b01,
                    // Southeast corner
                    index + 0b00,
                    index + 0b10,
                    index + 0b11,
                ];

                IntoIter::new(tile_indices)
            })
            .collect();

        RegionMesh { indices }
    }
}

impl From<RegionMesh> for Mesh {
    fn from(value: RegionMesh) -> Self {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(Indices::U32(value.indices)));
        mesh
    }
}
