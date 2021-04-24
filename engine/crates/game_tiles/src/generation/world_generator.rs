use crate::{Region, RegionWorldPosition};
use game_lib::bevy::ecs::component::Component;
use std::fmt::Debug;

pub trait WorldGenerator: Component + Debug {
    fn populate_region(&mut self, region_position: RegionWorldPosition, region: &mut Region);
}
