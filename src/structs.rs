/*!
Input/output world-related data structures for the algorithm
*/

use std::fmt::Display;
use std::ops::Add;
use std::ops::Mul;

/**
A cubic zone of the world, for which to run an extraction
*/
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BlockDims {
    /// Lowest x,y,z point
    pub base: [f32; 3],
    /// Side of the cube
    pub size: f32,
}

/**
A [Block] with attached number of subdivisions for the extraction
With n subdivisions, the block will contain n cells, encompassing n + 1 voxels across each dimension
*/
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Block {
    /// The zone
    pub dims: BlockDims,
    /// How many subdivisions
    pub subdivisions: usize,
}

impl Block {
    /// Shortcut constructor
    pub fn from(base: [f32; 3], size: f32, subdivisions: usize) -> Self {
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
pub struct Mesh {
    /// Flat vector of the vertex positions. Each consecutive three floats define x,y,z for one vertex
    pub positions: Vec<f32>,
    /// Flat vector of the vertex normals. Each consecutive three floats define x,y,z for one vertex
    pub normals: Vec<f32>,
    /**
    Flat vector of the triangle indices. Each consecutive i,j,k define one triangle by 3 indices.
    Indices are referring to the `positions` and `normals` "triples", so each index is in 0..positions.len()
    */
    pub triangle_indices: Vec<usize>,
}

impl Mesh {
    /// Shothand to get the triangles count
    pub fn num_tris(&self) -> usize {
        self.triangle_indices.len() / 3
    }
    /// Outputs a copy of triangles in a structured format
    pub fn tris(&self) -> Vec<Triangle> {
        let mut tris: Vec<Triangle> = vec![];
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
pub struct Triangle {
    /// Vertices
    pub vertices: [Vertex; 3],
}

/// A vertex, mostly for debugging or test purposes
#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Vertex {
    /// XYZ
    pub position: [f32; 3],
    /// XYZ of the normal
    pub normal: [f32; 3],
}

impl Display for Triangle {
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
pub struct Position {
    /// X
    pub x: f32,
    /// Y
    pub y: f32,
    /// Z
    pub z: f32,
}

impl Position {
    /// Interpolate between this `self` position and `other`, by the given `factor` (0 giving self, 1 giving other)
    pub fn interp_toward(&self, other: &Position, factor: f32) -> Position {
        Position {
            x: self.x + factor * (other.x - self.x),
            y: self.y + factor * (other.y - self.y),
            z: self.z + factor * (other.z - self.z),
        }
    }
}

impl Mul<f32> for &Position {
    type Output = Position;

    fn mul(self, rhs: f32) -> Self::Output {
        Position {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl Add<&[f32; 3]> for &Position {
    type Output = Position;

    fn add(self, rhs: &[f32; 3]) -> Self::Output {
        Position {
            x: self.x + rhs[0],
            y: self.y + rhs[1],
            z: self.z + rhs[2],
        }
    }
}
