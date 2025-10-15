/*!
Main mesh extraction methods
*/

use super::traits::mesh_builder::*;
use crate::implementation::algorithm;
use crate::structs::block::Block;
use crate::structs::block_star_view::BlockStarView;
use crate::traits::coordinate::Coordinate;
use crate::traits::voxel_data::VoxelData;
use crate::structs::transition_sides::TransitionSides;
use crate::structs::voxel_blocks::{VoxelBlockRelayingToField, VoxelVecBlock};
use crate::traits::data_field::DataField;
use crate::traits::voxel_block::{VoxelBlock};


/**
Extracts an iso-surface mesh

Arguments:
 * `blocks`: view of the block that has to be extracted and its data
 * `threshold`: density value defining the iso-surface
 * `mesh_builder`: builder object on which functions will be called to append vertices and triangles
 * The provided mesh_builder is handed back when the function returns.
 */
pub fn extract<C, V, M, B1, B2>(
    blocks: &BlockStarView<C, V, B1, B2>,
    threshold: V::Density,
    mesh_builder: M,
) -> M
where
    C: Coordinate,
    V: VoxelData,
    B1: VoxelBlock<C, V>,
    B2: VoxelBlock<C, V>,
    M: MeshBuilder<V, C>,
{
    algorithm::Extractor::new(blocks, threshold, mesh_builder).extract()
}


/**
How the field will be cached during a single extraction.
There is no case for caching "everything", as that's not a realistic use case: if B2 is a higher-res neighbour of central block B1, you will presumably also call the extraction for B2 as the central block. And caching of B2 should thus be done (manually) outside the extraction function. See tests for examples of how such caching is done.
 */
pub enum FieldCaching {
    /// No caching will happen. The field could (and probably will) be queried several times for the same voxel.
    CacheNothing,
    /**
    The central block (see [BlockStarView]) will be preemptively cached: the field will be accessed once for each voxel.
    The neighbour blocks will not be cached at all. Only some of their voxels will be accessed (at the edge of the central block), but potentially several times each.
    */
    CacheCentralBlockOnly,
}


/**
Extracts an iso-surface mesh for a [DataField]

Arguments:
 * `field`: the source of data
 * `caching_strategy`: how to cache the field. See [FieldCaching].
 * `block`: the block that has to be extracted
 * `transition_sides`: which sides of the block have to be rendered at higher resolution
 * `threshold`: density value defining the iso-surface
 * `mesh_builder`: builder object on which functions will be called to append vertices and triangles
 * The provided mesh_builder is returned back at the end.
 */
pub fn extract_from_field<C, V, M, F: DataField<V, C>>(
    // field: &dyn DataField<V, C>,
    field: &F,
    caching_strategy: FieldCaching,
    block: Block<C>,
    transition_sides: TransitionSides,
    threshold: V::Density,
    mesh_builder: M,
) -> M
where
    C: Coordinate,
    V: VoxelData + Copy,
    M: MeshBuilder<V, C>,
{
    match caching_strategy {
        FieldCaching::CacheNothing => {
            let blocks = BlockStarView::new_relaying_to_field(field, block, &transition_sides);
            algorithm::Extractor::new(&blocks, threshold, mesh_builder).extract()
        }
        FieldCaching::CacheCentralBlockOnly => {
            let central = VoxelVecBlock::cache(field, block);
            let mut blocks = BlockStarView::new_simple(central);
            for side in transition_sides {
                blocks = blocks.with_neighbour(VoxelBlockRelayingToField { field, block }, side);
            }
            algorithm::Extractor::new(&blocks, threshold, mesh_builder).extract()
        }
    }
}