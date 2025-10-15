/*!
Structs for world position
*/

use std::fmt::Debug;
use std::ops::Add;
use std::ops::Mul;

use crate::traits::coordinate::Coordinate;

/// A world space position
#[derive(Debug)]
pub struct SamplingPosition<C: Coordinate> {
    /// X
    pub x: C,
    /// Y
    pub y: C,
    /// Z
    pub z: C,
}

/// A world space position
#[derive(Debug, Eq, PartialEq)]
pub struct OutputPosition<C: Coordinate> {
    /// X
    pub x: C,
    /// Y
    pub y: C,
    /// Z
    pub z: C,
}

impl<C: Coordinate> OutputPosition<C> {
    /// Interpolate between this `self` position and `other`, by the given `factor` (0 giving self, 1 giving other)
    pub fn interpolate_toward(&self, other: &OutputPosition<C>, factor: C) -> OutputPosition<C> {
        OutputPosition {
            x: self.x + factor * (other.x - self.x),
            y: self.y + factor * (other.y - self.y),
            z: self.z + factor * (other.z - self.z),
        }
    }
}

impl<C> Mul<C> for &OutputPosition<C>
where
    C: Coordinate,
{
    type Output = OutputPosition<C>;

    fn mul(self, rhs: C) -> Self::Output {
        OutputPosition {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl<C> Add<&[C; 3]> for &OutputPosition<C>
where
    C: Coordinate,
{
    type Output = OutputPosition<C>;

    fn add(self, rhs: &[C; 3]) -> Self::Output {
        OutputPosition {
            x: self.x + rhs[0],
            y: self.y + rhs[1],
            z: self.z + rhs[2],
        }
    }
}
