use game_tiles::{Tile, TileWorldPosition};

pub trait WorldGenerator {
    fn get(&self, position: TileWorldPosition) -> Option<Tile>;
}
