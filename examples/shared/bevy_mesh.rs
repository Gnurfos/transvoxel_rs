/*!
This module contains mesh builders to produce [Bevy](https://bevyengine.org/) meshes.
*/

use bevy::asset::RenderAssetUsages;
use bevy::render::mesh::Mesh;
use bevy::render::render_resource::PrimitiveTopology::{LineList, TriangleList};
use transvoxel::{mesh_builder::*, traits::*};

#[derive(Default)]
pub struct BevyMeshBuilder {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub triangle_indices: Vec<u32>,
    vertices: usize,
}

/// A simple bevy mesh builder that:
///  - only populates position/normal attributes
///  - only looks at density of the VoxelData
impl BevyMeshBuilder {
    /**
    Build a Bevy mesh, producing a triangle list mesh with positions and normals
    from our mesh, but UV coordinates all set to 0
    */
    pub fn build(self) -> Mesh {
        let mut bevy_mesh = Mesh::new(TriangleList, RenderAssetUsages::default());
        let indices = bevy::render::mesh::Indices::U32(self.triangle_indices);
        bevy_mesh.insert_indices(indices);
        bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
        return bevy_mesh;
    }
    /**
    Convert to a Bevy mesh lines list, with positions and normals
    from our mesh, but UV coordinates all set to 0.
    Lines shared between 2 triangles are repeated, for implementation simplicity.
    */
    pub fn build_wireframe(self) -> Mesh {
        let mut bevy_mesh = Mesh::new(LineList, RenderAssetUsages::default());
        let tris_count = self.triangle_indices.len() / 3;
        let indices = (0..tris_count)
            .map(|i| vec![3 * i, 3 * i + 1, 3 * i + 1, 3 * i + 2, 3 * i + 2, 3 * i])
            .flatten()
            .map(|j| self.triangle_indices[j] as u32)
            .collect();
        let indices = bevy::render::mesh::Indices::U32(indices);
        bevy_mesh.insert_indices(indices);
        bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
        return bevy_mesh;
    }
}

impl<V> MeshBuilder<V, f32> for BevyMeshBuilder
where
    V: VoxelData<Density = f32>,
{
    fn add_vertex_between(
        &mut self,
        point_a: GridPoint<V, f32>,
        point_b: GridPoint<V, f32>,
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
        self.positions.push([position.x, position.y, position.z]);
        self.normals.push(normal);
        let index = self.vertices;
        self.vertices += 1;
        return VertexIndex(index);
    }

    fn add_triangle(
        &mut self,
        vertex_1_index: VertexIndex,
        vertex_2_index: VertexIndex,
        vertex_3_index: VertexIndex,
    ) {
        self.triangle_indices.push(vertex_1_index.0 as u32);
        self.triangle_indices.push(vertex_2_index.0 as u32);
        self.triangle_indices.push(vertex_3_index.0 as u32);
    }
}
