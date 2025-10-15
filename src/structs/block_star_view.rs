#![allow(missing_docs)]
/*!
A view on a main [VoxelBlock] and some of its neighbour blocks having a higher resolution
*/

use std::marker::PhantomData;

use crate::{structs::transition_sides::{TransitionSide, TransitionSides}, traits::{coordinate::Coordinate, voxel_data::VoxelData}};
use crate::structs::block::Block;
use crate::structs::voxel_blocks::VoxelBlockRelayingToField;
use crate::traits::data_field::DataField;
use crate::traits::voxel_block::{VoxelBlock};

/// A view on a central [VoxelBlock] and up to 6 of its neighbours, having double the resolution of the central block
pub struct BlockStarView<
    C: Coordinate,
    V: VoxelData,
    CentralBlock: VoxelBlock<C, V>,
    DenserNeighbourBlock: VoxelBlock<C, V>
> {
    central: CentralBlock,
    neighbours: [Option<DenserNeighbourBlock>; 6],
    transition_sides: TransitionSides,
    _v: PhantomData<V>,
    _c: PhantomData<C>,
}

impl<
    C: Coordinate,
    V: VoxelData,
    CentralBlock: VoxelBlock<C, V>,
    DenserNeighbourBlock: VoxelBlock<C, V>
> BlockStarView<C, V, CentralBlock, DenserNeighbourBlock> {

    pub fn central(&self) -> &CentralBlock {
        &self.central
    }

    pub fn transition_sides(&self) -> &TransitionSides {
        &self.transition_sides
    }

    pub fn neighbour_at(&self, side: TransitionSide) -> &DenserNeighbourBlock {
        let Some(ref block) = self.neighbours[Self::neighbour_index(side)] else {
            todo!("resultify the whole lib")
        };
        block
    }
    fn neighbour_index(side: TransitionSide) -> usize { // TODO result
        side as usize
    }
}

impl<'a, C: Coordinate, V: VoxelData> BlockStarView<C, V, VoxelBlockRelayingToField<'a, C, V>, VoxelBlockRelayingToField<'a, C, V>> {
    pub fn new_relaying_to_field(
        field: &'a dyn DataField<V, C>,
        block: Block<C>,
        transition_sides: &TransitionSides,
    ) -> BlockStarView<C, V, VoxelBlockRelayingToField<'a, C, V>, VoxelBlockRelayingToField<'a, C, V>> { //VoxelBlockRelayingToField<'a, C, V>, _> {
        let central = VoxelBlockRelayingToField {
            field,
            block,
        };
        let mut neighbours = [None, None, None, None, None, None];
        for side in transition_sides.into_iter() {
            let neighbour_block = block.high_res_neighbour_to(side);
            neighbours[side as usize] = Some(VoxelBlockRelayingToField {
                field,
                block: neighbour_block,
            });
        }
        BlockStarView {
            central,
            neighbours,
            transition_sides: *transition_sides,
            _v: PhantomData,
            _c: PhantomData,
        }
    }

}

impl<C: Coordinate, V: VoxelData, B1: VoxelBlock<C, V>, B2: VoxelBlock<C, V>> BlockStarView<C, V, B1, B2> {
    pub fn new_simple(central: B1) -> Self {
        Self {
            central,
            neighbours: [None, None, None, None, None, None],
            transition_sides: TransitionSide::none(),
            _v: PhantomData,
            _c: PhantomData,
        }
    }
    pub fn with_neighbour(mut self, neighbour: B2, side: TransitionSide) -> Self {
        self.neighbours[Self::neighbour_index(side)] = Some(neighbour);
        self.transition_sides |= side;
        self
    }
}

