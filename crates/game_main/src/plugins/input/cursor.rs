use bevy::prelude::*;

#[derive(Clone, Debug, Default)]
pub struct CursorState {
    pub screen_position: Vec2,
    pub world_position: Vec3,
}
