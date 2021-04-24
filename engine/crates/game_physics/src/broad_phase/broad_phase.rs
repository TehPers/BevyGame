use crate::bodies::AxisAlignedBoundingBox;
use game_lib::bevy::math::Vec2;

pub trait BroadPhase<'a> {
    type Id;
    type QueryBounds: IntoIterator<Item = Self::Id> + 'a;
    type QueryPoint: IntoIterator<Item = Self::Id> + 'a;

    fn query_bounds<'b: 'a>(&'b self, bounds: AxisAlignedBoundingBox) -> Self::QueryBounds;
    fn query_point<'b: 'a>(&'b self, point: Vec2) -> Self::QueryPoint;
}
