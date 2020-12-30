use crate::plugins::tiles::{Grid, GridCoordinate, Tile, TileSheetIndex};
use anyhow::Context;
use bevy::{
    prelude::*,
    render::{mesh::Indices, pipeline::PrimitiveTopology},
};
use std::convert::TryInto;
use tracing::instrument;

pub const TILE_SHEET_ROW_WIDTH: u32 = 16;
pub const TILE_SHEET_ROWS: u32 = 16;

#[derive(Debug)]
pub struct TileWorld {
    tiles: Grid<Option<Tile>>,
}

impl TileWorld {
    pub fn new(width: u32, height: u32) -> anyhow::Result<Self> {
        let tiles = Grid::new_default(width, height).context("failed to create tile grid")?;
        Ok(TileWorld { tiles })
    }

    pub fn get(&self, x: GridCoordinate, y: GridCoordinate) -> anyhow::Result<Option<Tile>> {
        self.tiles
            .get(x, y)
            .map(|&tile| tile)
            .context("failure getting tile")
    }

    pub fn get_mut(
        &mut self,
        x: GridCoordinate,
        y: GridCoordinate,
    ) -> anyhow::Result<&mut Option<Tile>> {
        self.tiles.get_mut(x, y).context("failure getting tile")
    }

    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (GridCoordinate, GridCoordinate, Option<Tile>)> + '_ {
        self.tiles.iter().map(|(x, y, tile)| (x, y, *tile))
    }

    pub fn iter_mut(
        &mut self,
    ) -> impl Iterator<Item = (GridCoordinate, GridCoordinate, &mut Option<Tile>)> {
        self.tiles.iter_mut()
    }

    pub fn iter_tiles(&self) -> impl Iterator<Item = (GridCoordinate, GridCoordinate, Tile)> + '_ {
        self.iter()
            .filter_map(|(x, y, tile)| tile.map(move |tile| (x, y, tile)))
    }

    pub fn iter_meshes(&self) -> impl Iterator<Item = TileMesh> + '_ {
        self.iter_tiles().map(|(x, y, tile)| {
            const TILE_SIZE: f32 = 1.0 / TILE_SHEET_ROW_WIDTH as f32;
            let index: TileSheetIndex = tile.into();
            let (u, v) = index.into_uv(TILE_SHEET_ROW_WIDTH, TILE_SHEET_ROWS);
            TileMesh {
                tile,
                vertices: [
                    [x as f32 - 0.5, y as f32 - 0.5],
                    [x as f32 - 0.5, y as f32 + 0.5],
                    [x as f32 + 0.5, y as f32 + 0.5],
                    [x as f32 + 0.5, y as f32 - 0.5],
                ],
                indices: [[0, 2, 1], [0, 3, 2]],
                uvs: [
                    [u, v + TILE_SIZE],
                    [u, v],
                    [u + TILE_SIZE, v],
                    [u + TILE_SIZE, v + TILE_SIZE],
                ],
            }
        })
    }
}

#[derive(Clone, Debug)]
pub struct TileMesh {
    tile: Tile,
    vertices: [[f32; 2]; 4],
    indices: [[u32; 3]; 2],
    uvs: [[f32; 2]; 4],
}

impl From<&TileWorld> for Mesh {
    #[instrument(skip(world))]
    fn from(world: &TileWorld) -> Self {
        // TODO: fetch vertices rather than push (add vertices as a uniform vec3[4])
        //       and push vertex index (u32) and tile position (vec2) instead
        // TODO: cull tiles that won't be visible
        // TODO: support chunking

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        let mut indices = Vec::new();
        let mut positions = Vec::new();
        let mut uvs = Vec::new();
        let mut colors: Vec<[f32; 4]> = Vec::new();
        for tile_mesh in world.iter_meshes() {
            let base_index: u32 = positions.len().try_into().unwrap();
            for &[v_x, v_y] in tile_mesh.vertices.iter() {
                positions.push([v_x, v_y, 0.0]);
                colors.push(Color::WHITE.into());
            }

            for &uv in tile_mesh.uvs.iter() {
                uvs.push(uv);
            }

            // Push indices
            for &[first, second, third] in tile_mesh.indices.iter() {
                indices.push(base_index + first);
                indices.push(base_index + second);
                indices.push(base_index + third);
            }
        }

        mesh.set_indices(Some(Indices::U32(indices)));
        mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.set_attribute("Tile_Color", colors);
        mesh
    }
}
