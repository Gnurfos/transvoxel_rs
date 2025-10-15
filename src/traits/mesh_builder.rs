/*!
Trait used to customize mesh generation
*/


use crate::structs::grid_point::GridPoint;
use crate::structs::vertex_index::VertexIndex;
use crate::traits::coordinate::Coordinate;
use crate::traits::voxel_data::VoxelData;


/// Trait you need to implement to build a mesh
pub trait MeshBuilder<V: VoxelData, C: Coordinate> {
    /// Called by the extraction algorithm when a new vertex it to be created between 2 grid points.
    ///
    /// Must return the index in the vertex buffer of the created vertex, as this will potentially get reused later.
    /// `interpolate_toward_b` indicates where the vertex is to be placed within the AB segment: near 0 means near A, near 1 means near B.
    fn add_vertex_between(
        &mut self,
        point_a: GridPoint<V, C>,
        point_b: GridPoint<V, C>,
        interpolate_toward_b: V::Density,
    ) -> VertexIndex;

    /// Called by the extraction algorithm when a triangle is to be created, using 3 previously created vertices.
    fn add_triangle(
        &mut self,
        vertex_1_index: VertexIndex,
        vertex_2_index: VertexIndex,
        vertex_3_index: VertexIndex,
    );
}
