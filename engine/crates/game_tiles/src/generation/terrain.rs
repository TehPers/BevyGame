use crate::{generation::WorldGenerator, Region, RegionWorldPosition, Tile, TileWorldPosition};
use game_lib::rand::Rng;
use std::{convert::TryInto, f32::consts::TAU};

#[derive(Clone, Copy, PartialEq, Debug)]
struct Wave {
    amplitude: f32,
    wavelength: f32,
    phase: f32,
}

#[derive(Debug)]
pub struct TerrainWorldGenerator {
    terrain_waves: Vec<Wave>,
}

impl TerrainWorldGenerator {
    pub fn new_random<R: Rng>(rand: &mut R) -> Self {
        // mountains:
        // let terrain_waves = (0..rand.gen_range(3..=6))
        //     .map(|_| Wave {
        //         amplitude: rand.gen_range(5.0..=20.0),
        //         wavelength: rand.gen_range(50.0..=250.0),
        //         phase: rand.gen_range(0.0..=1000.0),
        //     })
        //     .collect();

        let terrain_waves = (0..rand.gen_range(10..=20))
            .map(|_| Wave {
                amplitude: rand.gen_range(0.5..=5.0),
                wavelength: rand.gen_range(50.0..=500.0),
                phase: rand.gen_range(0.0..=1000.0),
            })
            .collect();

        TerrainWorldGenerator { terrain_waves }
    }
}

impl WorldGenerator for TerrainWorldGenerator {
    fn populate_region(&mut self, region_position: RegionWorldPosition, region: &mut Region) {
        let world_position: TileWorldPosition = region_position.into();
        for x in (0..Region::WIDTH.into()).map(|x| x + world_position.x) {
            let terrain_height = self.terrain_waves.iter().fold(100.0, |acc, wave| {
                let Wave {
                    amplitude,
                    wavelength,
                    phase,
                } = wave;

                let x = x as f32;
                let offset = amplitude * f32::sin(TAU / wavelength * x + phase);
                acc + offset
            }) as i32;

            // Stone
            for y in 0..terrain_height - 10 {
                let region_position = TileWorldPosition::new(x, y) - world_position;
                if let Ok(region_position) = region_position.try_into() {
                    if let Ok(tile) = region.get_mut(region_position) {
                        *tile = Some(Tile::Stone);
                    }
                }
            }

            // Dirt
            for y in (terrain_height - 10)..terrain_height {
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
