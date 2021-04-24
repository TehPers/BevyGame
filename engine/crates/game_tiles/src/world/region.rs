use std::{convert::TryInto, num::TryFromIntError};

use crate::{Tile, TileRegionCoordinate, TileRegionPosition, TileRegionRect, TileWorldPosition};
use game_lib::{
    bevy::{math::Vec2, prelude::*},
    derive_more::{Display, Error},
};
use game_morton::Morton;

// TODO: implement Serialize/Deserialize, doesn't support const generics yet so
//       array is a pain
// TODO: implement Reflect once support for arrays is added
#[derive(Clone, Debug)]
pub struct Region {
    tiles: [Option<Tile>; Self::TILES],
}

impl Default for Region {
    fn default() -> Self {
        Region {
            tiles: [None; Self::TILES],
        }
    }
}

impl Region {
    pub const WIDTH: TileRegionCoordinate = 16;
    pub const HEIGHT: TileRegionCoordinate = 16;
    pub const TILES: usize = Self::WIDTH as usize * Self::HEIGHT as usize;
    pub const BOUNDS: TileRegionRect = TileRegionRect::new(
        TileRegionPosition::ZERO,
        TileRegionPosition::new(Self::WIDTH, Self::HEIGHT),
    );

    pub fn get(&self, position: TileRegionPosition) -> Result<&Option<Tile>, RegionGetError> {
        self.tiles
            .get(Self::encode_pos(position)?)
            .ok_or(RegionGetError::OutOfBounds(position))
    }

    pub fn get_mut(
        &mut self,
        position: TileRegionPosition,
    ) -> Result<&mut Option<Tile>, RegionGetError> {
        self.tiles
            .get_mut(Self::encode_pos(position)?)
            .ok_or(RegionGetError::OutOfBounds(position))
    }

    fn encode_pos(position: TileRegionPosition) -> Result<usize, RegionGetError> {
        if position.x >= Self::WIDTH || position.y >= Self::HEIGHT {
            Err(RegionGetError::OutOfBounds(position))
        } else {
            Ok(Morton::encode_2d(position.x, position.y).into())
        }
    }

    fn decode_pos(index: usize) -> Result<TileRegionPosition, TryFromIntError> {
        let index = index.try_into()?;
        let (x, y) = Morton::decode_2d(index);
        Ok(TileRegionPosition::new(x, y))
    }

    pub fn iter(&self) -> impl Iterator<Item = (TileRegionPosition, &Option<Tile>)> {
        self.tiles
            .iter()
            .enumerate()
            .map(|(index, tile)| (Region::decode_pos(index).unwrap(), tile))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (TileRegionPosition, &mut Option<Tile>)> {
        self.tiles
            .iter_mut()
            .enumerate()
            .map(|(index, tile)| (Region::decode_pos(index).unwrap(), tile))
    }

    pub fn iter_rect(
        &self,
        rect: TileRegionRect,
    ) -> impl Iterator<Item = (TileRegionPosition, Result<&Option<Tile>, RegionGetError>)> {
        rect.iter_positions()
            .map(move |position| (position, self.get(position)))
    }

    pub fn iter_intersecting(
        &self,
        bottom_left: Vec2,
        top_right: Vec2,
    ) -> impl Iterator<Item = (TileRegionPosition, Result<&Option<Tile>, RegionGetError>)> {
        let bottom_left: TileRegionPosition = bottom_left.max(Vec2::ZERO).floor().into();
        let top_right: TileRegionPosition = top_right.max(Vec2::ZERO).ceil().into();
        self.iter_rect(TileRegionRect::new(bottom_left, top_right - bottom_left))
    }
}

#[derive(Clone, Debug, Display, Error)]
pub enum RegionGetError {
    #[display(fmt = "coordinates are out of bounds: {}", _0)]
    OutOfBounds(#[error(ignore)] TileRegionPosition),

    #[display(fmt = "failed to encode tile coordinates into an index: {}", position)]
    IntConversion {
        position: TileRegionPosition,
        #[error(source)]
        source: TryFromIntError,
    },
}

#[derive(Clone, Debug, Display, Error)]
pub enum GetTileError {
    #[display(fmt = "coordinates are out of bounds: {}", _0)]
    OutOfBounds(#[error(ignore)] TileWorldPosition),

    #[display(fmt = "failed to encode tile coordinates into an index: {}", position)]
    IntConversion {
        position: TileWorldPosition,
        #[error(source)]
        source: TryFromIntError,
    },
}
