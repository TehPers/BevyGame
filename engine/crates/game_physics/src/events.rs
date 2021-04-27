use game_lib::bevy::prelude::*;
use game_tiles::{EntityWorldPosition, Tile, TileWorldPosition};

/// A collision between an entity and a tile.
#[derive(Clone, Debug, Reflect)]
pub struct TileCollision {
    pub entity: Entity,
    pub entity_velocity: EntityWorldPosition,
    pub axis: TileCollisionAxis,
    pub tile: Tile,
    pub tile_position: TileWorldPosition,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Reflect)]
pub enum TileCollisionAxis {
    X,
    Y,
}

/// A collision between two bodies.
#[derive(Clone, Debug, Reflect)]
pub struct EntityCollision {
    /// Entities involved with the collision.
    pub entities: (Entity, Entity),
}
