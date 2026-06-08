use std::f32::consts::FRAC_PI_2;

use derive_more::Constructor;
use geo::{Contains, Triangle, coord};
use glam::Vec2;
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Archive, Deserialize, Serialize, PartialEq, Eq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct Xy(pub u64, pub u64);

impl From<Xy> for (u64, u64) {
    fn from(value: Xy) -> Self {
        (value.0, value.1)
    }
}

impl From<(isize, isize)> for Xy {
    fn from((x, y): (isize, isize)) -> Self {
        Self(x as u64, y as u64)
    }
}

impl From<(isize, isize, isize)> for Xy {
    fn from((x, y, _): (isize, isize, isize)) -> Self {
        Self(x as u64, y as u64)
    }
}

impl Xy {
    pub fn distance(self, other: Self) -> f32 {
        ((other.0 as f32 - self.0 as f32).powi(2) + (other.1 as f32 - self.1 as f32).powi(2)).sqrt()
    }
}

#[derive(
    Debug,
    Copy,
    Clone,
    Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct Angle(pub f32);

impl Angle {
    pub fn from_points(to_point: &Vec2, from_point: &Vec2) -> Self {
        Self(f32::atan2(to_point.y - from_point.y, to_point.x - from_point.x) + FRAC_PI_2)
    }

    pub fn zero() -> Self {
        Self(0.)
    }
}

impl std::ops::Add for Angle {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl std::ops::Neg for Angle {
    type Output = Self;

    fn neg(self) -> Self {
        Self(-self.0)
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Rect {
    /// X coordinate of the left edge of the rect.
    pub x: f32,
    /// Y coordinate of the top edge of the rect.
    pub y: f32,
    /// Total width of the rect
    pub w: f32,
    /// Total height of the rect.
    pub h: f32,
}

impl Rect {
    /// Create a new `Rect`.
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub fn to_array(&self) -> [f32; 4] {
        [self.x, self.y, self.w, self.h]
    }

    pub fn from_array(values: [f32; 4]) -> Self {
        Self {
            x: values[0],
            y: values[1],
            w: values[2],
            h: values[3],
        }
    }
}

pub struct Shape {
    pub top_left: Vec2,
    pub top_right: Vec2,
    pub bottom_right: Vec2,
    pub bottom_left: Vec2,
}

impl Shape {
    pub fn from_rect(rect: &Rect) -> Self {
        Self {
            top_left: Vec2::new(rect.x, rect.y),
            top_right: Vec2::new(rect.x + rect.w, rect.y),
            bottom_right: Vec2::new(rect.x + rect.w, rect.y + rect.h),
            bottom_left: Vec2::new(rect.x, rect.y + rect.h),
        }
    }

    pub fn rotate(&self, angle: &Angle) -> Self {
        let width = self.top_right.x - self.top_left.x;
        let height = self.bottom_left.y - self.top_left.y;
        let center_offset = Vec2::new(width / 2., height / 2.);
        let reference_point = self.top_left + center_offset;

        let after_top_left = apply_angle_on_point(&self.top_left, &reference_point, angle);
        let after_top_right = apply_angle_on_point(&self.top_right, &reference_point, angle);
        let after_bottom_right = apply_angle_on_point(&self.bottom_right, &reference_point, angle);
        let after_bottom_left = apply_angle_on_point(&self.bottom_left, &reference_point, angle);

        Self {
            top_left: after_top_left,
            top_right: after_top_right,
            bottom_right: after_bottom_right,
            bottom_left: after_bottom_left,
        }
    }

    pub fn from_point(&self, point: Vec2) -> Self {
        let width = self.top_right.x - self.top_left.x;
        let height = self.bottom_left.y - self.top_left.y;

        Self {
            top_left: point,
            top_right: point + Vec2::new(width, 0.),
            bottom_right: point + Vec2::new(width, height),
            bottom_left: point + Vec2::new(0., height),
        }
    }

    pub fn centered(&self) -> Self {
        let width = self.top_right.x - self.top_left.x;
        let height = self.bottom_left.y - self.top_left.y;

        Self {
            top_left: self.top_left - Vec2::new(width / 2., height / 2.),
            top_right: self.top_right - Vec2::new(width / 2., height / 2.),
            bottom_right: self.bottom_right - Vec2::new(width / 2., height / 2.),
            bottom_left: self.bottom_left - Vec2::new(width / 2., height / 2.),
        }
    }

    pub fn contains(&self, point: &Vec2) -> bool {
        let triangle1 = Triangle::new(
            coord! { x: self.top_left.x, y: self.top_left.y },
            coord! { x: self.top_right.x, y: self.top_right.y },
            coord! { x: self.bottom_left.x, y: self.bottom_left.y },
        );
        let triangle2 = Triangle::new(
            coord! { x: self.bottom_right.x, y: self.bottom_right.y },
            coord! { x: self.bottom_left.x, y: self.bottom_left.y },
            coord! { x: self.top_right.x, y: self.top_right.y },
        );

        triangle1.contains(&coord! { x: point.x, y: point.y })
            || triangle2.contains(&coord! { x: point.x, y: point.y })
    }
}

pub fn apply_angle_on_point(point_to_rotate: &Vec2, reference_point: &Vec2, angle: &Angle) -> Vec2 {
    let sin = f32::sin(angle.0);
    let cos = f32::cos(angle.0);
    let pt = (
        point_to_rotate.x - reference_point.x,
        point_to_rotate.y - reference_point.y,
    );
    let rotated = (
        reference_point.x + pt.0 * cos - pt.1 * sin,
        reference_point.y + pt.0 * sin + pt.1 * cos,
    );
    Vec2::new(rotated.0, rotated.1)
}

#[derive(
    Debug,
    Clone,
    Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Constructor,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(
    Debug,
    Clone,
    Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Constructor,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct Direction {
    pub x: f32,
    pub y: f32,
}

impl Direction {
    pub const NORTH: Self = Self::new(0., 1.);
}

impl From<Direction> for Vec2 {
    fn from(value: Direction) -> Self {
        Vec2::new(value.x, value.y)
    }
}

impl From<Vec2> for Direction {
    fn from(value: Vec2) -> Self {
        Direction::new(value.x, value.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance_line_x() {
        // Given
        let a = Xy(0, 0);
        let b = Xy(1, 0);

        // When
        let distance = a.distance(b);

        // Then
        assert_eq!(distance, 1.0);
    }

    #[test]
    fn test_distance_line_x_neg() {
        // Given
        let a = Xy(1, 0);
        let b = Xy(0, 0);

        // When
        let distance = a.distance(b);

        // Then
        assert_eq!(distance, 1.0);
    }

    #[test]
    fn test_distance_line_y() {
        // Given
        let a = Xy(0, 0);
        let b = Xy(0, 1);

        // When
        let distance = a.distance(b);

        // Then
        assert_eq!(distance, 1.0);
    }

    #[test]
    fn test_distance_diag() {
        // Given
        let a = Xy(0, 0);
        let b = Xy(1, 1);

        // When
        let distance = a.distance(b);

        // Then
        assert_eq!(distance, 1.4142135);
    }
}
