#![allow(missing_docs)]
/*!
Block structure
*/

use crate::{structs::transition_sides::TransitionSide, traits::coordinate::Coordinate};
use crate::implementation::voxel_coordinates::RegularVoxelIndex;
use crate::prelude::TransitionSides;
use crate::structs::position::{OutputPosition, SamplingPosition};

/**
A [Block] (cubic region of the world) with attached number of subdivisions for the extraction.
With n subdivisions, the block will contain n^3 cells, encompassing n + 1 voxels across each dimension.
```
use transvoxel::prelude::*;
// Just meant to be constructed and passed around
let a_block = Block {
    base: [10.0, 20.0, 30.0],
    size: 10.0,
    subdivisions: 8,
};
let another_block = Block::new([10.0, 20.0, 30.0], 10.0, 8);
```
*/
#[derive(Clone, Copy)]
pub struct Block<C>
where
    C: Coordinate,
{
    /// Lowest x,y,z point
    pub base: [C; 3],
    /// Side of the cube
    pub size: C,
    /// How many subdivisions
    pub subdivisions: usize,
}

impl<C> Block<C>
where
    C: Coordinate,
{
    pub fn new(base: [C; 3], size: C, subdivisions: usize) -> Self {
        Block {
            base,
            size,
            subdivisions,
        }
    }
}

impl<C: Coordinate> Block<C> {
    pub fn high_res_neighbour_to(&self, side: TransitionSide) -> Self {
        let size = self.size;
        let base = match side {
            TransitionSide::LowX => [self.base[0] - size, self.base[1], self.base[2]],
            TransitionSide::HighX => [self.base[0] + size, self.base[1], self.base[2]],
            TransitionSide::LowY => [self.base[0], self.base[1] - size, self.base[2]],
            TransitionSide::HighY => [self.base[0], self.base[1] + size, self.base[2]],
            TransitionSide::LowZ => [self.base[0], self.base[1], self.base[2] - size],
            TransitionSide::HighZ => [self.base[0], self.base[1], self.base[2] + size],
        };
        let subdivisions = self.subdivisions * 2;
        Self { base, size, subdivisions }
    }

    /// Position of a given voxel, on the original regular grid for the block
    /// (for field sampling purposes)
    pub fn original_voxel_position(&self, index: RegularVoxelIndex) -> SamplingPosition<C> {
        let x = self.base[0]
            + self.size * C::from_ratio(index.x, self.subdivisions);
        let y = self.base[1]
            + self.size * C::from_ratio(index.y, self.subdivisions);
        let z = self.base[2]
            + self.size * C::from_ratio(index.z, self.subdivisions);
        SamplingPosition { x, y, z }
    }

    /// Position of a given voxel, after it has been potentially moved by "shrinking".
    /// (for output purposes).
    pub fn morphed_voxel_position(&self, index: RegularVoxelIndex, transition_sides: &TransitionSides) -> OutputPosition<C> {
        let SamplingPosition { mut x, mut y, mut z } = self.original_voxel_position(index);
        shrink_if_needed(
            &mut x, &mut y, &mut z,
            index.x, index.y, index.z,
            self.subdivisions,
            transition_sides,
            self.size,
        );
        OutputPosition { x, y, z }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn shrink_if_needed<C: Coordinate>(
    x: &mut C,
    y: &mut C,
    z: &mut C,
    xi: isize,
    yi: isize,
    zi: isize,
    subdivisions: usize,
    transition_sides: &TransitionSides,
    block_size: C,
) {
    let cell_size = block_size * C::from_ratio(1, subdivisions);
    let shrink: C = C::shrink_factor() * cell_size;
    if can_shrink(xi, yi, zi, subdivisions, transition_sides) {
        if (xi == 0) && (transition_sides.contains(TransitionSide::LowX)) {
            *x = *x + shrink;
        } else if (xi as usize == subdivisions)
            && (transition_sides.contains(TransitionSide::HighX))
        {
            *x = *x - shrink;
        }
        if (yi == 0) && (transition_sides.contains(TransitionSide::LowY)) {
            *y = *y + shrink;
        } else if (yi as usize == subdivisions)
            && (transition_sides.contains(TransitionSide::HighY))
        {
            *y = *y - shrink;
        }
        if (zi == 0) && (transition_sides.contains(TransitionSide::LowZ)) {
            *z = *z + shrink;
        } else if (zi as usize == subdivisions)
            && (transition_sides.contains(TransitionSide::HighZ))
        {
            *z = *z - shrink;
        }
    }
}

// Do not shrink grid point (in any direction) if it's close to a face where the other block is rendered at the same
// (or lower) level of details (ie: not a transition side)
fn can_shrink(
    xi: isize,
    yi: isize,
    zi: isize,
    subdivisions: usize,
    transition_sides: &TransitionSides,
) -> bool {
    let dont_shrink = ((xi == 0) && !transition_sides.contains(TransitionSide::LowX))
        || ((xi == subdivisions as isize) && !transition_sides.contains(TransitionSide::HighX))
        || ((yi == 0) && !transition_sides.contains(TransitionSide::LowY))
        || ((yi == subdivisions as isize) && !transition_sides.contains(TransitionSide::HighY))
        || ((zi == 0) && !transition_sides.contains(TransitionSide::LowZ))
        || ((zi == subdivisions as isize) && !transition_sides.contains(TransitionSide::HighZ));
    !dont_shrink
}
