use std::convert::TryInto;

use crate::{generation::WorldGenerator, Region, RegionWorldPosition, Tile, TileWorldPosition};
use game_lib::{
    bevy::utils::HashMap,
    derive_more::{Display, Error},
    tracing::trace_span,
};

#[derive(Debug)]
pub struct GameWorld {
    regions: HashMap<RegionWorldPosition, Region>,
    generator: Box<dyn WorldGenerator>,
}

impl GameWorld {
    pub fn new(generator: Box<dyn WorldGenerator>) -> Self {
        GameWorld {
            regions: HashMap::default(),
            generator,
        }
    }

    pub fn get_tile(
        &self,
        position: TileWorldPosition,
    ) -> Result<&Option<Tile>, GameWorldGetError> {
        let region_world_position = position.into();
        let region = self.get_region(region_world_position)?;

        let tile_region_position = position - TileWorldPosition::from(region_world_position);
        Ok(region.get(tile_region_position.try_into().unwrap()).unwrap())
    }

    pub fn get_tile_mut(
        &mut self,
        position: TileWorldPosition,
    ) -> Result<&mut Option<Tile>, GameWorldGetError> {
        let region_world_position = position.into();
        let region = self.get_region_mut(region_world_position)?;

        let tile_region_position = position - TileWorldPosition::from(region_world_position);
        Ok(region.get_mut(tile_region_position.try_into().unwrap()).unwrap())
    }

    pub fn get_or_generate_tile(&mut self, position: TileWorldPosition) -> &mut Option<Tile> {
        let region_world_position = position.into();
        let region = self.get_or_generate_region(region_world_position);

        let tile_region_position = position - TileWorldPosition::from(region_world_position);
        region.get_mut(tile_region_position.try_into().unwrap()).unwrap()
    }

    pub fn get_region(&self, position: RegionWorldPosition) -> Result<&Region, GameWorldGetError> {
        self.regions
            .get(&position)
            .ok_or(GameWorldGetError::NotYetGenerated)
    }

    pub fn get_region_mut(
        &mut self,
        position: RegionWorldPosition,
    ) -> Result<&mut Region, GameWorldGetError> {
        self.regions
            .get_mut(&position)
            .ok_or(GameWorldGetError::NotYetGenerated)
    }

    pub fn get_or_generate_region(&mut self, position: RegionWorldPosition) -> &mut Region {
        let GameWorld {
            ref mut regions,
            ref mut generator,
            ..
        } = self;

        regions.entry(position).or_insert_with(|| {
            trace_span!("region_generation", %position).in_scope(|| {
                let mut region = Region::default();
                generator.populate_region(position, &mut region);
                region
            })
        })
    }
}

#[derive(Clone, Debug, Display, Error)]
pub enum GameWorldGetError {
    #[display(fmt = "the requested item has not been generated yet")]
    NotYetGenerated,
}
