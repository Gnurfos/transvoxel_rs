/*!
Structs and traits used to customize mesh generation
*/

use std::fmt::Debug;
use std::ops::Add;
use std::ops::Mul;

use crate::traits::Coordinate;
use crate::traits::VoxelData;

/// A world space position
#[derive(Debug)]
pub struct Position<C: Coordinate> {
    /// X
    pub x: C,
    /// Y
    pub y: C,
    /// Z
    pub z: C,
}

impl<C: Coordinate> Position<C> {
    /// Interpolate between this `self` position and `other`, by the given `factor` (0 giving self, 1 giving other)
    pub fn interp_toward(&self, other: &Position<C>, factor: C) -> Position<C> {
        Position {
            x: self.x + factor * (other.x - self.x),
            y: self.y + factor * (other.y - self.y),
            z: self.z + factor * (other.z - self.z),
        }
    }
}

impl<C> Mul<C> for &Position<C>
where
    C: Coordinate,
{
    type Output = Position<C>;

    fn mul(self, rhs: C) -> Self::Output {
        Position {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl<C> Add<&[C; 3]> for &Position<C>
where
    C: Coordinate,
{
    type Output = Position<C>;

    fn add(self, rhs: &[C; 3]) -> Self::Output {
        Position {
            x: self.x + rhs[0],
            y: self.y + rhs[1],
            z: self.z + rhs[2],
        }
    }
}

/// A grid point on the voxel grid. A pair of these will be passed to the mesh generator,
/// when creating vertices
#[derive(Debug)]
pub struct GridPoint<V: VoxelData, C: Coordinate> {
    /// World location of the grid point
    pub position: Position<C>,
    /// Density gradient (estimated) at the grid point
    pub gradient: (V::Density, V::Density, V::Density),
    /// Data at the grid point that was obtained from the field
    pub voxel_data: V,
}

/// An index in the vertex buffer
#[derive(Default, Clone, Copy)]
pub struct VertexIndex(pub usize);

/// Trait you need to implement to build a mesh
pub trait MeshBuilder<V: VoxelData, C: Coordinate> {
    /// Called by the extraction algorithm when a new vertex it to be created between 2 grid points.
    ///
    /// Must return the index in the vertex buffer of the created vertex, as this will potentially get reused later.
    /// `interp_toward_b` indicates where the vertex is to be placed within the AB segment: near 0 means near A, near 1 means near B.
    fn add_vertex_between(
        &mut self,
        point_a: GridPoint<V, C>,
        point_b: GridPoint<V, C>,
        interp_toward_b: V::Density,
    ) -> VertexIndex;

    /// Called by the extraction algorithm when a triangle is to be created, using 3 pre-created vertices.
    fn add_triangle(
        &mut self,
        vertex_1_index: VertexIndex,
        vertex_2_index: VertexIndex,
        vertex_3_index: VertexIndex,
    );
}
