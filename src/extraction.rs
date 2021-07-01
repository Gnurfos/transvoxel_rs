/*!
Main mesh extraction methods
*/
use std::cell::RefCell;

use super::implementation::algorithm::Extractor;
use super::structs::*;
use super::{
    density::*,
    voxel_source::{VoxelSource, WorldMappingVoxelSource},
};
use crate::transition_sides::TransitionSides;

/**
Extracts an iso-surface [Mesh] for a [VoxelSource]

Arguments:
 * `source`: the density source
 * `block`: the world zone for which to extract, and its subdivisions count
 * `threshold`: density value defining the iso-surface
 * `transition_sides`: the set of sides of the block which need to be adapted to neighbour double-resolution blocks (twice the subdivisions)
 */
pub fn extract<D, S>(
    source: S,
    block: &Block<D::F>,
    threshold: D,
    transition_sides: TransitionSides,
) -> Mesh<D::F>
where
    D: Density,
    S: VoxelSource<D>,
{
    Extractor::new(source, block, threshold, transition_sides).extract()
}

/**
Extracts an iso-surface [Mesh] for a [ScalarField]

Arguments:
 * `field`: the density field
 * `block`: the world zone for which to extract, and its subdivisions count
 * `threshold`: density value defining the iso-surface
 * `transition_sides`: the set of sides of the block which need to be adapted to neighbour double-resolution blocks (twice the subdivisions)
*/
pub fn extract_from_field<D, FIELD>(
    field: FIELD,
    block: &Block<D::F>,
    threshold: D,
    transition_sides: TransitionSides,
) -> Mesh<D::F>
where
    D: Density,
    FIELD: ScalarField<D, D::F>, // TODO simplify, only need 1 parameter
{
    let mut source = WorldMappingVoxelSource { field, block };
    Extractor::new(&mut source, block, threshold, transition_sides).extract()
}

/**
Extracts an iso-surface [Mesh] for a [ScalarField]-compatible closure

Arguments:
 * `f`: the closure providing world densities
 * `block`: the world zone for which to extract, and its subdivisions count
 * `threshold`: density value defining the iso-surface
 * `transition_sides`: the set of sides of the block which need to be adapted to neighbour double-resolution blocks (twice the subdivisions)
*/
pub fn extract_from_fn<D, FUN>(
    f: FUN,
    block: &Block<D::F>,
    threshold: D,
    transition_sides: TransitionSides,
) -> Mesh<D::F>
where
    D: Density,
    FUN: Fn(D::F, D::F, D::F) -> D,
{
    let field = ScalarFieldForFn(f);
    let mut source = WorldMappingVoxelSource { field, block };
    Extractor::new(&mut source, block, threshold, transition_sides).extract()
}

/**
Same as  [extract_from_fn] for mutable closures
*/
pub fn extract_from_fnmut<D, FUN>(
    f: FUN,
    block: &Block<D::F>,
    threshold: D,
    transition_sides: TransitionSides,
) -> Mesh<D::F>
where
    D: Density,
    FUN: FnMut(D::F, D::F, D::F) -> D,
{
    let field = ScalarFieldForFnMut(RefCell::new(f));
    let mut source = WorldMappingVoxelSource { field, block };
    Extractor::new(&mut source, block, threshold, transition_sides).extract()
}
