use super::super::transition_sides::TransitionSide::*;
use super::super::voxel_coordinates::{HighResolutionVoxelIndex, RegularVoxelIndex};
use hamcrest::prelude::*;
use hamcrest::*;

#[test]
fn index_conversion() {
    let this_block_size = 5;
    let i = HighResolutionVoxelIndex::from(LowZ, 2, 3, -1, 0, 1);
    assert_that!(
        i.to_higher_res_neighbour_block_index(this_block_size),
        equal_to(RegularVoxelIndex { x: 3, y: 6, z: 11 })
    );
}

#[test]
fn index_conversion2() {
    let this_block_size = 5;
    let i = HighResolutionVoxelIndex::from(HighX, 2, 3, -1, 0, 1);
    assert_that!(
        i.to_higher_res_neighbour_block_index(this_block_size),
        equal_to(RegularVoxelIndex { x: -1, y: 6, z: 3 })
    );
}

#[test]
fn index_conversion3() {
    let this_block_size = 5;
    let i = HighResolutionVoxelIndex::from(HighZ, 2, 3, 1, 1, -1);
    assert_that!(
        i.to_higher_res_neighbour_block_index(this_block_size),
        equal_to(RegularVoxelIndex { x: 5, y: 7, z: 1 })
    );
}
