use std::convert::TryInto;

use crate::{
    generation::WorldGenerator, Region, RegionWorldPosition, Tile, TileWorldCoordinate,
    TileWorldPosition,
};

#[derive(Clone, Copy, Debug, Hash)]
pub struct FlatWorldGenerator {
    fill: Tile,
    fill_height: Option<TileWorldCoordinate>,
}

impl FlatWorldGenerator {
    pub fn new(fill: Tile, fill_height: Option<TileWorldCoordinate>) -> Self {
        FlatWorldGenerator { fill, fill_height }
    }
}

impl WorldGenerator for FlatWorldGenerator {
    fn populate_region(&mut self, region_position: RegionWorldPosition, region: &mut Region) {
        match self.fill_height {
            None => {
                for position in Region::BOUNDS.iter_positions() {
                    if let Ok(tile) = region.get_mut(position) {
                        *tile = Some(self.fill);
                    }
                }
            }
            Some(fill_height) => {
                let world_position: TileWorldPosition = region_position.into();
                for x in (0..Region::WIDTH.into()).map(|x| x + world_position.x) {
                    for y in 0..fill_height {
                        let tile_region_position = TileWorldPosition::new(x, y) - world_position;
                        if let Ok(tile_region_position) = tile_region_position.try_into() {
                            if let Ok(tile) = region.get_mut(tile_region_position) {
                                *tile = Some(self.fill);
                            }
                        }
                    }
                }
            }
        }
    }
}
