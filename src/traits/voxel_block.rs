/*!
Trait defining a voxel block
*/

use std::rc::Rc;

use crate::structs::block::Block;
use crate::structs::voxel_index::VoxelIndex;
use crate::traits::{coordinate::Coordinate, voxel_data::VoxelData};

/**
 Something that provides access to voxel data stored in a given block,
 at that block's resolution.
*/
pub trait VoxelBlock<C: Coordinate, V: VoxelData> {
    /// The block
    fn block(&self) -> &Block<C>;
    /// Access data for one voxel of the block.
    fn get(&self, index: VoxelIndex) -> V;
}

/// A reference to a [VoxelBlock] also acts as a [VoxelBlock]
impl<C: Coordinate, V: VoxelData, B: VoxelBlock<C, V>> VoxelBlock<C, V> for &B {
    fn block(&self) -> &Block<C> {
        (*self).block()
    }


    fn get(&self, index: VoxelIndex) -> V {
        (*self).get(index)
    }
}

/// A [Rc] to a [VoxelBlock] also acts as a [VoxelBlock]
impl<C: Coordinate, V: VoxelData, B: VoxelBlock<C, V>> VoxelBlock<C, V> for Rc<B> {
    fn block(&self) -> &Block<C> {
        (**self).block()
    }


    fn get(&self, index: VoxelIndex) -> V {
        (**self).get(index)
    }
}
