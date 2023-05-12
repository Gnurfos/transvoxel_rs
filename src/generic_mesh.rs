/*!
A generic (engine independent) implementation of a Mesh, and an associated MeshBuilder
*/

use std::fmt::Debug;
use std::fmt::Display;

use num::Float;

use crate::mesh_builder::GridPoint;
use crate::mesh_builder::MeshBuilder;
use crate::mesh_builder::VertexIndex;
use crate::traits::Density;

/**
Mesh
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

/// A MeshBuilder that builds Mesh
pub struct GenericMeshBuilder<F>
where
    F: Float,
{
    positions: Vec<F>,
    normals: Vec<F>,
    triangle_indices: Vec<usize>,
    vertices: usize,
}

#[allow(clippy::new_without_default)]
impl<F> GenericMeshBuilder<F>
where
    F: Float,
{
    /// Create a fresh builder
    pub fn new() -> Self {
        Self {
            positions: vec![],
            normals: vec![],
            triangle_indices: vec![],
            vertices: 0,
        }
    }
    /// Output the Mesh
    pub fn build(self) -> Mesh<F> {
        Mesh {
            positions: self.positions,
            normals: self.normals,
            triangle_indices: self.triangle_indices,
        }
    }
}

impl<F> Mesh<F>
where
    F: Float,
{
    /// Shorthand to get the triangles count
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
        tris
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

impl MeshBuilder<f32, f32> for GenericMeshBuilder<f32> {
    fn add_vertex_between(
        &mut self,
        point_a: GridPoint<f32, f32>,
        point_b: GridPoint<f32, f32>,
        interp_toward_b: f32,
    ) -> VertexIndex {
        let position = point_a
            .position
            .interp_toward(&point_b.position, interp_toward_b);
        let gradient_x =
            point_a.gradient.0 + interp_toward_b * (point_b.gradient.0 - point_a.gradient.0);
        let gradient_y =
            point_a.gradient.1 + interp_toward_b * (point_b.gradient.1 - point_a.gradient.1);
        let gradient_z =
            point_a.gradient.2 + interp_toward_b * (point_b.gradient.2 - point_a.gradient.2);
        let normal = f32::gradients_to_normal(gradient_x, gradient_y, gradient_z);
        self.positions.push(position.x);
        self.positions.push(position.y);
        self.positions.push(position.z);
        self.normals.push(normal[0]);
        self.normals.push(normal[1]);
        self.normals.push(normal[2]);
        let index = self.vertices;
        self.vertices += 1;
        VertexIndex(index)
    }

    fn add_triangle(
        &mut self,
        vertex_1_index: VertexIndex,
        vertex_2_index: VertexIndex,
        vertex_3_index: VertexIndex,
    ) {
        self.triangle_indices.push(vertex_1_index.0);
        self.triangle_indices.push(vertex_2_index.0);
        self.triangle_indices.push(vertex_3_index.0);
    }
}
