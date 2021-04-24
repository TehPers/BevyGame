use crate::Region;
use game_lib::{
    bevy::{math::Vec2, reflect::Reflect},
    derive_more::{Add, AddAssign, Display, From, Into, Sub, SubAssign},
    serde::{Deserialize, Serialize},
};
use std::{
    convert::{TryFrom, TryInto},
    num::TryFromIntError,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg},
};

macro_rules! pos_type {
    (
        $(#[$pos_meta:meta])* $pos_name:ident,
        $(#[$rect_meta:meta])* $rect_name:ident,
        $(#[$coord_meta:meta])* $coord_name:ident = $coord_ty:ty,
        $zero:expr,
        $one:expr,
        [$($impl:tt),*]
    ) => {
        $(#[$coord_meta])*
        pub type $coord_name = $coord_ty;

        #[derive(
            Clone,
            Copy,
            PartialEq,
            Debug,
            Display,
            Default,
            From,
            Into,
            Add,
            AddAssign,
            Sub,
            SubAssign,
            Reflect,
            Serialize,
            Deserialize,
        )]
        #[display(fmt = "({}, {})", x, y)]
        #[serde(crate = "game_lib::serde")]
        $(#[$pos_meta])*
        pub struct $pos_name {
            pub x: $coord_name,
            pub y: $coord_name,
        }

        impl $pos_name {
            pub const ZERO: Self = Self::new($zero, $zero);
            pub const ONE: Self = Self::new($one, $one);
            pub const X: Self = Self::new($one, $zero);
            pub const Y: Self = Self::new($zero, $one);

            pub const fn new(x: $coord_name, y: $coord_name) -> Self {
                $pos_name { x, y }
            }

            pub fn offset(self, x: $coord_name, y: $coord_name) -> Self {
                $pos_name {
                    x: self.x + x,
                    y: self.y + y,
                }
            }
        }

        impl Mul<$coord_name> for $pos_name {
            type Output = Self;

            fn mul(mut self, value: $coord_name) -> Self {
                self.mul_assign(value);
                self
            }
        }

        impl MulAssign<$coord_name> for $pos_name {
            fn mul_assign(&mut self, value: $coord_name) {
                self.x *= value;
                self.y *= value;
            }
        }

        impl Mul<$pos_name> for $pos_name {
            type Output = Self;

            fn mul(mut self, value: $pos_name) -> Self {
                self.mul_assign(value);
                self
            }
        }

        impl MulAssign<$pos_name> for $pos_name {
            fn mul_assign(&mut self, value: $pos_name) {
                self.x *= value.x;
                self.y *= value.y;
            }
        }

        impl Div<$coord_name> for $pos_name {
            type Output = Self;

            fn div(mut self, value: $coord_name) -> Self {
                self.div_assign(value);
                self
            }
        }

        impl DivAssign<$coord_name> for $pos_name {
            fn div_assign(&mut self, value: $coord_name) {
                self.x /= value;
                self.y /= value;
            }
        }

        impl Div<$pos_name> for $pos_name {
            type Output = Self;

            fn div(mut self, value: $pos_name) -> Self {
                self.div_assign(value);
                self
            }
        }

        impl DivAssign<$pos_name> for $pos_name {
            fn div_assign(&mut self, value: $pos_name) {
                self.x /= value.x;
                self.y /= value.y;
            }
        }

        impl From<Vec2> for $pos_name {
            fn from(value: Vec2) -> Self {
                let x = value[0] as $coord_name;
                let y = value[1] as $coord_name;
                $pos_name::new(x, y)
            }
        }

        impl From<$pos_name> for Vec2 {
            fn from(position: $pos_name) -> Self {
                Vec2::new(position.x as f32, position.y as f32)
            }
        }

        #[derive(Clone, Copy, PartialEq, Debug, Default, Reflect, Serialize, Deserialize)]
        #[serde(crate = "game_lib::serde")]
        $(#[$rect_meta])*
        pub struct $rect_name {
            pub bottom_left: $pos_name,
            pub size: $pos_name,
        }

        impl $rect_name {
            pub const fn new(bottom_left: $pos_name, size: $pos_name) -> Self {
                $rect_name { bottom_left, size }
            }

            pub fn left(self) -> $coord_name {
                self.bottom_left.x
            }

            pub fn right(self) -> $coord_name {
                self.bottom_left.x + self.size.x
            }

            pub fn top(self) -> $coord_name {
                self.bottom_left.y + self.size.y
            }

            pub fn bottom(self) -> $coord_name {
                self.bottom_left.y
            }

            pub fn bottom_left(self) -> $pos_name {
                self.bottom_left
            }

            pub fn bottom_right(self) -> $pos_name {
                self.bottom_left + self.size * $pos_name::X
            }

            pub fn top_left(self) -> $pos_name {
                self.bottom_left + self.size * $pos_name::Y
            }

            pub fn top_right(self) -> $pos_name {
                self.bottom_left + self.size
            }

            pub fn size(self) -> $pos_name {
                self.size
            }

            pub fn expand(mut self, diameter: $coord_name) -> Self {
                self.bottom_left -= $pos_name::ONE * diameter;
                self.size += $pos_name::ONE * diameter * ($one + $one);
                self
            }

            pub fn offset(mut self, offset: $pos_name) -> Self {
                self.bottom_left += offset;
                self
            }
        }

        $(pos_type!(@impl $impl, $pos_name, $rect_name, $coord_name);)*
    };
    (@impl step, $pos_name:ident, $rect_name:ident, $coord_name:ident) => {
        impl $rect_name {
            pub fn iter_positions(self) -> impl Iterator<Item = $pos_name> + Clone {
                (self.bottom()..self.top()).flat_map(move |y| {
                    (self.left()..self.right()).map(move |x| $pos_name::new(x, y))
                })
            }
        }
    };
    (@impl signed, $pos_name:ident, $rect_name:ident, $coord_name:ident) => {
        impl $pos_name {
            pub fn signum(self) -> Self {
                $pos_name::new(self.x.signum(), self.y.signum())
            }
        }

        impl Neg for $pos_name {
            type Output = Self;

            fn neg(self) -> Self {
                $pos_name::new(-self.x, -self.y)
            }
        }
    };
    (@impl float, $pos_name:ident, $rect_name:ident, $coord_name:ident) => {
        impl $pos_name {
            pub fn is_finite(self) -> bool {
                self.x.is_finite() && self.y.is_finite()
            }
        }
    };
}

pos_type!(
    /// A global position within the world. Given that the world is a grid of
    /// tiles, this is a position of a tile within that grid.
    #[derive(Eq, Hash)]
    TileWorldPosition,
    /// A rectangle within the world.
    #[derive(Eq, Hash)]
    TileWorldRect,
    /// A global coordinate within the world.
    TileWorldCoordinate = i32,
    0,
    1,
    [step, signed]
);

pos_type!(
    /// A position within a region, relative to that region.
    #[derive(Eq, Hash)]
    TileRegionPosition,
    /// A rectangle within a region.
    #[derive(Eq, Hash)]
    TileRegionRect,
    /// A region-local coordinate.
    TileRegionCoordinate = u8,
    0,
    1,
    [step]
);

pos_type!(
    /// A position of a region within the world. Given that the world is a grid
    /// of regions, this is a position of a region within that grid.
    #[derive(Eq, Hash)]
    RegionWorldPosition,
    /// A rectangle which selects region within the world.
    #[derive(Eq, Hash)]
    RegionWorldRect,
    /// A coordinate of a region within the world. This follows a different
    /// scale than [WorldCoordinate] by being a coordinate given the world is
    /// split into [Region::Width] by [Region::HEIGHT] squares of tiles.
    RegionWorldCoordinate = i32,
    0,
    1,
    [step, signed]
);

pos_type!(
    /// A position of an entity within the world.
    EntityWorldPosition,
    /// A rectangle in the world using entity coordinates.
    EntityWorldRect,
    /// A coordinate of an entity within the world.
    EntityWorldCoordinate = f32,
    0.0,
    1.0,
    [signed, float]
);

impl From<TileRegionPosition> for TileWorldPosition {
    fn from(value: TileRegionPosition) -> Self {
        TileWorldPosition::new(value.x.into(), value.y.into())
    }
}

impl From<TileRegionRect> for TileWorldRect {
    fn from(value: TileRegionRect) -> Self {
        TileWorldRect::new(value.bottom_left.into(), value.size.into())
    }
}

impl From<RegionWorldPosition> for TileWorldPosition {
    fn from(value: RegionWorldPosition) -> Self {
        TileWorldPosition::new(
            value.x * TileWorldCoordinate::from(Region::WIDTH),
            value.y * TileWorldCoordinate::from(Region::HEIGHT),
        )
    }
}

impl From<RegionWorldRect> for TileWorldRect {
    fn from(value: RegionWorldRect) -> Self {
        TileWorldRect::new(
            value.bottom_left.into(),
            TileWorldPosition::new(
                value.size.x * TileWorldCoordinate::from(Region::WIDTH),
                value.size.y * TileWorldCoordinate::from(Region::HEIGHT),
            ),
        )
    }
}

impl From<TileWorldPosition> for RegionWorldPosition {
    fn from(value: TileWorldPosition) -> Self {
        RegionWorldPosition::new(
            value.x.div_euclid(TileWorldCoordinate::from(Region::WIDTH)),
            value
                .y
                .div_euclid(TileWorldCoordinate::from(Region::HEIGHT)),
        )
    }
}

impl From<TileWorldRect> for RegionWorldRect {
    fn from(value: TileWorldRect) -> Self {
        let offset = TileWorldPosition::new(
            TileWorldCoordinate::from(Region::WIDTH) - 1,
            TileWorldCoordinate::from(Region::HEIGHT) - 1,
        );

        let bottom_left = value.bottom_left.into();
        let top_right: RegionWorldPosition = (value.top_right() + offset).into();
        let size = top_right - bottom_left;
        RegionWorldRect::new(bottom_left, size)
    }
}

impl TryFrom<TileWorldPosition> for TileRegionPosition {
    type Error = TryFromIntError;

    fn try_from(TileWorldPosition { x, y }: TileWorldPosition) -> Result<Self, Self::Error> {
        Ok(TileRegionPosition::new(x.try_into()?, y.try_into()?))
    }
}

impl From<EntityWorldPosition> for TileWorldPosition {
    fn from(value: EntityWorldPosition) -> Self {
        TileWorldPosition::new(
            value.x as TileWorldCoordinate,
            value.y as TileWorldCoordinate,
        )
    }
}

impl From<EntityWorldRect> for TileWorldRect {
    fn from(value: EntityWorldRect) -> Self {
        let bottom_left = value.bottom_left.into();
        let top_right = value.top_right();
        let top_right = TileWorldPosition::new(
            top_right.x.ceil() as TileWorldCoordinate,
            top_right.y.ceil() as TileWorldCoordinate,
        );
        TileWorldRect::new(bottom_left, top_right - bottom_left)
    }
}

impl Add<TileRegionPosition> for TileWorldPosition {
    type Output = Self;

    fn add(mut self, rhs: TileRegionPosition) -> Self::Output {
        self += rhs;
        self
    }
}

impl AddAssign<TileRegionPosition> for TileWorldPosition {
    fn add_assign(&mut self, rhs: TileRegionPosition) {
        self.x += i32::from(rhs.x);
        self.y += i32::from(rhs.y);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_position_to_world_region_position() {
        fn check(
            expected_x: RegionWorldCoordinate,
            expected_y: RegionWorldCoordinate,
            world_x: TileWorldCoordinate,
            world_y: TileWorldCoordinate,
        ) {
            let world_position = TileWorldPosition::new(world_x, world_y);
            let expected = RegionWorldPosition::new(expected_x, expected_y);
            assert_eq!(expected, RegionWorldPosition::from(world_position));
        }

        check(0, 0, 0, 0);
        check(0, 0, 1, 1);
        check(-1, -1, -1, -1);
        check(4, 3, (Region::WIDTH * 4).into(), (Region::WIDTH * 3).into());
        check(-2, -1, -(TileWorldCoordinate::from(Region::WIDTH) + 1), -4);
    }

    #[test]
    fn world_rect_to_world_region_rect() {
        fn check(expected: RegionWorldRect, world_rect: TileWorldRect) {
            assert_eq!(expected, RegionWorldRect::from(world_rect));
        }

        check(
            RegionWorldRect::new(RegionWorldPosition::ZERO, RegionWorldPosition::ONE),
            TileWorldRect::new(TileWorldPosition::ZERO, TileWorldPosition::ONE),
        );
        check(
            RegionWorldRect::new(RegionWorldPosition::ZERO, RegionWorldPosition::ONE),
            TileWorldRect::new(
                TileWorldPosition::ZERO,
                TileWorldPosition::new(Region::WIDTH.into(), Region::HEIGHT.into()),
            ),
        );
        check(
            RegionWorldRect::new(
                RegionWorldPosition::new(-2, -1),
                RegionWorldPosition::new(3, 3),
            ),
            TileWorldRect::new(
                TileWorldPosition::new(-(TileWorldCoordinate::from(Region::WIDTH) + 1), -1),
                TileWorldPosition::new(
                    TileWorldCoordinate::from(Region::WIDTH) * 2,
                    TileWorldCoordinate::from(Region::HEIGHT) * 2,
                ),
            ),
        )
    }
}
