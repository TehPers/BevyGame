use std::ops::{Add, AddAssign, Sub, SubAssign};

use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct AxisAlignedBoundingBox {
    pub top_left: Vec2,
    pub size: Vec2,
}

impl AxisAlignedBoundingBox {
    pub fn new(top_left: Vec2, size: Vec2) -> Self {
        AxisAlignedBoundingBox { top_left, size }
    }

    pub fn from_center(center: Vec2, size: Vec2) -> Self {
        AxisAlignedBoundingBox {
            top_left: center - size / 2.0,
            size,
        }
    }

    pub fn from_corners(top_left: Vec2, bottom_right: Vec2) -> Self {
        AxisAlignedBoundingBox {
            top_left,
            size: bottom_right - top_left,
        }
    }

    /// Returns the smallest AABB that contains all the given child AABBs. This
    /// returns `None` only if no AABBs are given.
    pub fn from_children(aabbs: &[AxisAlignedBoundingBox]) -> Option<Self> {
        let (top_left, bottom_right) = aabbs.iter().fold(None, |corners, cur| match corners {
            None => Some((cur.top_left(), cur.bottom_right())),
            Some((top_left, bottom_right)) => Some((
                top_left.min(cur.top_left()),
                bottom_right.max(cur.bottom_right()),
            )),
        })?;
        Some(AxisAlignedBoundingBox::from_corners(top_left, bottom_right))
    }

    pub fn left(&self) -> f32 {
        self.top_left[0]
    }

    pub fn right(&self) -> f32 {
        self.top_left[0] + self.size[0]
    }

    pub fn top(&self) -> f32 {
        self.top_left[1]
    }

    pub fn bottom(&self) -> f32 {
        self.top_left[1] + self.size[1]
    }

    pub fn top_left(&self) -> Vec2 {
        self.top_left
    }

    pub fn top_right(&self) -> Vec2 {
        self.top_left + self.size * Vec2::unit_x()
    }

    pub fn bottom_left(&self) -> Vec2 {
        self.top_left + self.size * Vec2::unit_y()
    }

    pub fn bottom_right(&self) -> Vec2 {
        self.top_left + self.size
    }

    pub fn center(&self) -> Vec2 {
        self.top_left + self.size / 2.0
    }

    pub fn size(&self) -> Vec2 {
        self.size
    }

    pub fn contains(&self, other: &AxisAlignedBoundingBox) -> bool {
        self.left() <= other.left()
            && self.right() >= other.right()
            && self.top() <= other.top()
            && self.bottom() >= other.bottom()
    }

    pub fn contains_point(&self, point: Vec2) -> bool {
        point[0] >= self.left()
            && point[0] < self.right()
            && point[1] >= self.top()
            && point[0] < self.bottom()
    }

    pub fn intersects(&self, other: &AxisAlignedBoundingBox) -> bool {
        self.left() < other.right()
            && self.right() > other.left()
            && self.top() < other.bottom()
            && self.bottom() > other.top()
    }

    pub fn intersection(
        &self,
        other: &AxisAlignedBoundingBox,
        epsilon: f32,
    ) -> Option<AxisAlignedBoundingBox> {
        let left = self.left().max(other.left());
        let right = self.right().min(other.right());
        let top = self.top().max(other.top());
        let bottom = self.bottom().min(other.bottom());
        let width = right - left;
        let height = bottom - top;

        if width > epsilon && height > epsilon {
            Some(AxisAlignedBoundingBox {
                top_left: Vec2::new(left, top),
                size: Vec2::new(width, height),
            })
        } else {
            None
        }
    }
}

impl Add<Vec2> for AxisAlignedBoundingBox {
    type Output = Self;
    fn add(mut self, rhs: Vec2) -> Self::Output {
        self += rhs;
        self
    }
}

impl AddAssign<Vec2> for AxisAlignedBoundingBox {
    fn add_assign(&mut self, rhs: Vec2) {
        self.top_left += rhs;
    }
}

impl Sub<Vec2> for AxisAlignedBoundingBox {
    type Output = Self;
    fn sub(mut self, rhs: Vec2) -> Self::Output {
        self -= rhs;
        self
    }
}

impl SubAssign<Vec2> for AxisAlignedBoundingBox {
    fn sub_assign(&mut self, rhs: Vec2) {
        self.top_left -= rhs;
    }
}
