use bevy::prelude::*;

#[derive(Clone, Debug, Default, Reflect)]
pub struct CursorState {
    pub screen_position: Vec2,
    pub world_position: Vec2,
}
