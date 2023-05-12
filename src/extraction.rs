/*!
Main mesh extraction methods
*/

use super::implementation::algorithm::Extractor;
use super::mesh_builder::*;
use super::traits::*;
use super::voxel_source::*;
use crate::transition_sides::TransitionSides;

/**
Extracts an iso-surface mesh for a [VoxelSource]

Arguments:
 * `source`: the voxel data source
 * `block`: the world zone for which to extract, and its subdivisions count
 * `threshold`: density value defining the iso-surface
 * `transition_sides`: the set of sides of the block which need to be adapted to neighbour double-resolution blocks (twice the subdivisions)
 * `mesh_builder`: builder object on which functions will be called to append vertices and triangles
 * The provided mesh_builder is returned back at the end.
 */
pub fn extract<C, V, S, M>(
    source: S,
    block: &Block<C>,
    threshold: V::Density,
    transition_sides: TransitionSides,
    mesh_builder: M,
) -> M
where
    C: Coordinate,
    V: VoxelData,
    S: VoxelSource<V>,
    M: MeshBuilder<V, C>,
{
    Extractor::new(source, block, threshold, transition_sides, mesh_builder).extract()
}

/**
Extracts an iso-surface mesh for a [DataField]

Arguments:
 * `field`: the voxel data field
 * `block`: the world zone for which to extract, and its subdivisions count
 * `threshold`: density value defining the iso-surface
 * `transition_sides`: the set of sides of the block which need to be adapted to neighbour double-resolution blocks (twice the subdivisions)
 * `mesh_builder`: builder object on which functions will be called to append vertices and triangles
 * The provided mesh_builder is returned back at the end.
 */
pub fn extract_from_field<C, V, FIELD, M>(
    field: FIELD,
    block: &Block<C>,
    threshold: V::Density,
    transition_sides: TransitionSides,
    mesh_builder: M,
) -> M
where
    C: Coordinate,
    V: VoxelData,
    FIELD: DataField<V, C>,
    M: MeshBuilder<V, C>,
{
    let source = WorldMappingVoxelSource { field, block };
    Extractor::new(source, block, threshold, transition_sides, mesh_builder).extract()
}

/**
Extracts an iso-surface mesh for a [DataField]-compatible closure

Arguments:
 * `f`: the closure providing world data
 * `block`: the world zone for which to extract, and its subdivisions count
 * `threshold`: density value defining the iso-surface
 * `transition_sides`: the set of sides of the block which need to be adapted to neighbour double-resolution blocks (twice the subdivisions)
 * `mesh_builder`: builder object on which functions will be called to append vertices and triangles
 * The provided mesh_builder is returned back at the end.
*/
pub fn extract_from_fn<C, V, FUN, M>(
    field: FUN,
    block: &Block<C>,
    threshold: V::Density,
    transition_sides: TransitionSides,
    mesh_builder: M,
) -> M
where
    C: Coordinate,
    V: VoxelData,
    FUN: FnMut(C, C, C) -> V,
    M: MeshBuilder<V, C>,
{
    let source = WorldMappingVoxelSource { field, block };
    Extractor::new(source, block, threshold, transition_sides, mesh_builder).extract()
}
