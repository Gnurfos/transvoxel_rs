/*!
Input/output world-related data structures for the algorithm
*/

use std::fmt::Debug;
use std::fmt::Display;
use std::ops::Add;
use std::ops::Mul;

use num::Float;

/**
A cubic zone of the world, for which to run an extraction
*/
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BlockDims<F>
where F: Float
{
    /// Lowest x,y,z point
    pub base: [F; 3],
    /// Side of the cube
    pub size: F,
}

/**
A [Block] with attached number of subdivisions for the extraction
With n subdivisions, the block will contain n cells, encompassing n + 1 voxels across each dimension
```
# use transvoxel::structs::*;
// Just meant to be constructed and passed around
let a_block = Block {
    dims: BlockDims {
        base: [10.0, 20.0, 30.0],
        size: 10.0,
    },
    subdivisions: 8,
};
let another_block = Block::from([10.0, 20.0, 30.0], 10.0, 8);
```
*/
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Block<F>
where F: Float,
{
    /// The zone
    pub dims: BlockDims<F>,
    /// How many subdivisions
    pub subdivisions: usize,
}

impl<F> Block<F>
where F: Float
{
    /// Shortcut constructor
    pub fn from(base: [F; 3], size: F, subdivisions: usize) -> Self {
        Block {
            dims: BlockDims {
                base: base,
                size: size,
            },
            subdivisions: subdivisions,
        }
    }
}

/**
Output mesh for the algorithm
*/
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Mesh<F>
where
    F: Float,
{
    /// Flat vector of the vertex positions. Each consecutive three floats define x,y,z for one vertex
    pub positions: Vec<F>,
    /// Flat vector of the vertex normals. Each consecutive three floats define x,y,z for one vertex
    pub normals: Vec<F>,
    /**
    Flat vector of the triangle indices. Each consecutive i,j,k define one triangle by 3 indices.
    Indices are referring to the `positions` and `normals` "triples", so each index is in 0..positions.len()
    */
    pub triangle_indices: Vec<usize>,
}

impl<F> Mesh<F>
where
    F: Float,
{
    /// Shothand to get the triangles count
    pub fn num_tris(&self) -> usize {
        self.triangle_indices.len() / 3
    }
    /// Outputs a copy of triangles in a structured format
    pub fn tris(&self) -> Vec<Triangle<F>> {
        let mut tris: Vec<Triangle<F>> = vec![];
        for i in 0..self.num_tris() {
            let i1 = self.triangle_indices[3 * i];
            let i2 = self.triangle_indices[3 * i + 1];
            let i3 = self.triangle_indices[3 * i + 2];
            tris.push(Triangle {
                vertices: [
                    Vertex {
                        position: [
                            self.positions[3 * i1],
                            self.positions[3 * i1 + 1],
                            self.positions[3 * i1 + 2],
                        ],
                        normal: [
                            self.normals[3 * i1],
                            self.normals[3 * i1 + 1],
                            self.normals[3 * i1 + 2],
                        ],
                    },
                    Vertex {
                        position: [
                            self.positions[3 * i2],
                            self.positions[3 * i2 + 1],
                            self.positions[3 * i2 + 2],
                        ],
                        normal: [
                            self.normals[3 * i2],
                            self.normals[3 * i2 + 1],
                            self.normals[3 * i2 + 2],
                        ],
                    },
                    Vertex {
                        position: [
                            self.positions[3 * i3],
                            self.positions[3 * i3 + 1],
                            self.positions[3 * i3 + 2],
                        ],
                        normal: [
                            self.normals[3 * i3],
                            self.normals[3 * i3 + 1],
                            self.normals[3 * i3 + 2],
                        ],
                    },
                ],
            });
        }
        return tris;
    }
}

/// A triangle, mostly for debugging or test purposes
#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Triangle<F>
where
    F: Float,
{
    /// Vertices
    pub vertices: [Vertex<F>; 3],
}

/// A vertex, mostly for debugging or test purposes
#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Vertex<F>
where
    F: Float,
{
    /// XYZ
    pub position: [F; 3],
    /// XYZ of the normal
    pub normal: [F; 3],
}

impl<F> Display for Triangle<F>
where
    F: Float + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Triangle:")?;
        let [v1, v2, v3] = self.vertices;
        writeln!(f, "    + Pos {:?}  Norm {:?}", v1.position, v1.normal)?;
        writeln!(f, "    + Pos {:?}  Norm {:?}", v2.position, v2.normal)?;
        writeln!(f, "    + Pos {:?}  Norm {:?}", v3.position, v3.normal)?;
        Ok(())
    }
}

/// A world space position
pub struct Position<F> {
    /// X
    pub x: F,
    /// Y
    pub y: F,
    /// Z
    pub z: F,
}

impl<F> Position<F>
where F: Float
{
    /// Interpolate between this `self` position and `other`, by the given `factor` (0 giving self, 1 giving other)
    pub fn interp_toward(&self, other: &Position<F>, factor: F) -> Position<F> {
        Position {
            x: self.x + factor * (other.x - self.x),
            y: self.y + factor * (other.y - self.y),
            z: self.z + factor * (other.z - self.z),
        }
    }
}

impl<F> Mul<F> for &Position<F>
where F: Float
{
    type Output = Position<F>;

    fn mul(self, rhs: F) -> Self::Output {
        Position {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl<F> Add<&[F; 3]> for &Position<F>
where F: Float
{
    type Output = Position<F>;

    fn add(self, rhs: &[F; 3]) -> Self::Output {
        Position {
            x: self.x + rhs[0],
            y: self.y + rhs[1],
            z: self.z + rhs[2],
        }
    }
}
