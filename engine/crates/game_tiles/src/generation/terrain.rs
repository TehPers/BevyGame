use crate::{
    generation::{Waves, WavesConfig, WorldGenerator},
    Region, RegionWorldPosition, Tile, TileWorldPosition,
};
use game_lib::rand::Rng;
use std::convert::TryInto;

#[derive(Debug)]
pub struct TerrainWorldGenerator {
    terrain_waves: Waves,
    dirt_waves: Waves,
}

impl TerrainWorldGenerator {
    pub fn new_random<R: Rng>(rand: &mut R) -> Self {
        // mountains:
        // let terrain_waves = Waves::new_rand(rand, WavesConfig {
        //     waves: 3..=6,
        //     amplitude: 5.0..=20.0,
        //     wavelength: 50.0..=250.0,
        //     phase: 0.0..=1000.0,
        // });

        let terrain_waves = Waves::new_rand(
            rand,
            WavesConfig {
                waves: 10..=20,
                amplitude: 0.5..=5.0,
                wavelength: 50.0..=500.0,
                phase: 0.0..=1000.0,
            },
        );

        let dirt_waves = Waves::new_rand(
            rand,
            WavesConfig {
                waves: 5..=10,
                amplitude: 1.0..=1.5,
                wavelength: 100.0..=500.0,
                phase: 0.0..=1000.0,
            },
        );

        TerrainWorldGenerator {
            terrain_waves,
            dirt_waves,
        }
    }
}

impl WorldGenerator for TerrainWorldGenerator {
    fn populate_region(&mut self, region_position: RegionWorldPosition, region: &mut Region) {
        let world_position: TileWorldPosition = region_position.into();
        for x in (0..Region::WIDTH.into()).map(|x| x + world_position.x) {
            let terrain_height = (100.0 + self.terrain_waves.get(x as f32)) as i32;
            let dirt_height = (10.0 + self.dirt_waves.get(x as f32)).max(0.0) as i32;

            // Stone
            for y in 0..terrain_height - dirt_height {
                let region_position = TileWorldPosition::new(x, y) - world_position;
                if let Ok(region_position) = region_position.try_into() {
                    if let Ok(tile) = region.get_mut(region_position) {
                        *tile = Some(Tile::Stone);
                    }
                }
            }

            // Dirt
            for y in (terrain_height - dirt_height)..terrain_height {
                let region_position = TileWorldPosition::new(x, y) - world_position;
                if let Ok(region_position) = region_position.try_into() {
                    if let Ok(tile) = region.get_mut(region_position) {
                        *tile = Some(Tile::Dirt);
                    }
                }
            }
        }
    }
}
