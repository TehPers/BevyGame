use bevy::{prelude::*, reflect::TypeUuid};
use derive_more::{Display, From, Into};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize, Reflect, TypeUuid)]
#[uuid = "8f1fead5-a9bc-4590-8789-dae63b32ad66"]
#[repr(u32)]
pub enum Tile {
    Stone,
    Dirt,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Display, Hash, From, Into)]
pub struct TileSheetIndex(u32);

impl TileSheetIndex {
    pub fn into_uv(self, row_width: u32, rows: u32) -> (f32, f32) {
        let x = self.0 % row_width;
        let y = self.0 / row_width;
        (x as f32 / row_width as f32, y as f32 / rows as f32)
    }
}

impl From<Tile> for TileSheetIndex {
    fn from(tile: Tile) -> Self {
        match tile {
            Tile::Stone => 0.into(),
            Tile::Dirt => 1.into(),
        }
    }
}
