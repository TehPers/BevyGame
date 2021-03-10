use game_tiles::{Tile, TilePosition};

pub trait WorldGenerator {
    fn get(&self, position: TilePosition) -> Option<Tile>;
}
