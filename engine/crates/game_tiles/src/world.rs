use crate::Tile;
use bevy::{math::Vec2, prelude::*};
use derive_more::{Add, Display, Error, From, Into, Sub};
use game_morton::Morton;
use std::{convert::TryInto, num::TryFromIntError};

/// Number of tiles in a row on the tilesheet.
pub const TILE_SHEET_ROW_LENGTH: u32 = 16;

/// Number of tiles in a column on the tilesheet.
pub const TILE_SHEET_COL_LENGTH: u32 = 16;

/// Side length of each tile as a percentage of the total texture size
pub const TILE_UV_SIZE: f32 = 1.0 / TILE_SHEET_ROW_LENGTH as f32;

pub type TileCoordinate = u32;

#[derive(
    Clone, Copy, PartialEq, Eq, Debug, Display, Hash, Default, From, Into, Add, Sub, Reflect,
)]
#[display(fmt = "({}, {})", x, y)]
pub struct TilePosition {
    pub x: TileCoordinate,
    pub y: TileCoordinate,
}

impl TilePosition {
    pub fn new(x: TileCoordinate, y: TileCoordinate) -> Self {
        TilePosition { x, y }
    }
}

impl From<Vec2> for TilePosition {
    fn from(value: Vec2) -> Self {
        let x = value[0].floor() as TileCoordinate;
        let y = value[1].floor() as TileCoordinate;
        TilePosition::new(x, y)
    }
}

impl From<TilePosition> for Vec2 {
    fn from(position: TilePosition) -> Self {
        Vec2::new(position.x as f32, position.y as f32)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default, Reflect)]
pub struct TileRegion {
    pub bottom_left: TilePosition,
    pub size: TilePosition,
}

impl TileRegion {
    pub fn new(bottom_left: TilePosition, size: TilePosition) -> Self {
        TileRegion { bottom_left, size }
    }

    pub fn left(&self) -> TileCoordinate {
        self.bottom_left.x
    }

    pub fn right(&self) -> TileCoordinate {
        self.bottom_left.x + self.size.x
    }

    pub fn top(&self) -> TileCoordinate {
        self.bottom_left.y + self.size.y
    }

    pub fn bottom(&self) -> TileCoordinate {
        self.bottom_left.y
    }

    pub fn size(&self) -> TilePosition {
        self.size
    }

    pub fn iter_positions(&self) -> impl Iterator<Item = TilePosition> + '_ {
        (self.bottom()..self.top())
            .flat_map(move |y| (self.left()..self.right()).map(move |x| TilePosition::new(x, y)))
    }
}

#[derive(Debug, Default, Reflect)]
pub struct TileWorld {
    width: TileCoordinate,
    height: TileCoordinate,
    tiles: Vec<Option<Tile>>,
}

impl TileWorld {
    pub fn new(width: TileCoordinate, height: TileCoordinate) -> Result<Self, TryFromIntError> {
        let size = Self::encode_pos((width, height).into())?;
        Ok(TileWorld {
            width,
            height,
            tiles: vec![None; size],
        })
    }

    fn encode_pos(position: TilePosition) -> Result<usize, TryFromIntError> {
        let index = Morton::encode_2d(position.x, position.y);
        index.try_into()
    }

    fn _decode_pos(index: usize) -> Result<TilePosition, TryFromIntError> {
        let index = index.try_into()?;
        let (x, y) = Morton::decode_2d(index);
        Ok(TilePosition::new(x, y))
    }

    pub fn size(&self) -> TilePosition {
        TilePosition::new(self.width, self.height)
    }

    pub fn get(&self, position: TilePosition) -> Result<Option<Tile>, GetTileError> {
        let index = Self::encode_pos(position)
            .map_err(|source| GetTileError::IntConversion { position, source })?;
        self.tiles
            .get(index)
            .copied()
            .ok_or(GetTileError::OutOfBounds(position))
    }

    pub fn get_mut(&mut self, position: TilePosition) -> Result<&mut Option<Tile>, GetTileError> {
        let index = Self::encode_pos(position)
            .map_err(|source| GetTileError::IntConversion { position, source })?;
        self.tiles
            .get_mut(index)
            .ok_or(GetTileError::OutOfBounds(position))
    }

    pub fn iter_rect(
        &self,
        region: TileRegion,
    ) -> impl Iterator<Item = (TilePosition, Result<Option<Tile>, GetTileError>)> + '_ {
        (region.bottom()..region.top())
            .flat_map(move |y| {
                (region.left()..region.right()).map(move |x| TilePosition::new(x, y))
            })
            .map(move |position| (position, self.get(position)))
    }

    pub fn iter_intersecting(
        &self,
        bottom_left: Vec2,
        top_right: Vec2,
    ) -> impl Iterator<Item = (TilePosition, Result<Option<Tile>, GetTileError>)> + '_ {
        let bottom_left: TilePosition = bottom_left.max(Vec2::zero()).floor().into();
        let top_right: TilePosition = top_right.max(Vec2::zero()).ceil().into();
        self.iter_rect(TileRegion::new(bottom_left, top_right - bottom_left))
    }
}

#[derive(Clone, Debug, Display, Error)]
pub enum GetTileError {
    #[display(fmt = "coordinates are out of bounds: {}", _0)]
    OutOfBounds(#[error(ignore)] TilePosition),

    #[display(fmt = "failed to encode tile coordinates into an index: {}", position)]
    IntConversion {
        position: TilePosition,
        #[error(source)]
        source: TryFromIntError,
    },
}
