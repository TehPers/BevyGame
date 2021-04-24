use game_lib::{
    bevy::prelude::*,
    derive_more::{Display, From, Into},
    serde::{Deserialize, Serialize},
};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize, Reflect)]
#[serde(crate = "game_lib::serde")]
pub enum Tile {
    Stone,
    Dirt,
}

impl Tile {
    pub fn index(self) -> TileSheetIndex {
        match self {
            Tile::Dirt => 0.into(),
            Tile::Stone => 1.into(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Display, Hash, From, Into)]
pub struct TileSheetIndex(pub u16);

impl TileSheetIndex {
    pub fn into_uv(self, row_width: u16, rows: u16) -> (f32, f32) {
        let x = self.0 % row_width;
        let y = self.0 / row_width;
        (x as f32 / row_width as f32, y as f32 / rows as f32)
    }
}

impl From<Tile> for TileSheetIndex {
    fn from(tile: Tile) -> Self {
        tile.index()
    }
}
