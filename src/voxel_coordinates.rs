/*!
Structs for addressing discrete voxels
 If you subdivide the world or a volume with a grid, "voxels" are the intersection points of the grids
 The cubes between them are called "cells"

"Regular" means: at the lower voxel resolution, defined by the block's subdivisions.
That is, without considering transition faces. Even when introducing a transition face
and shifting some low resolution voxels toward inside the block, we still call these
voxels "regular", as nothing changes about them in the algorithm, except their contribution
to output vertex positions. The density sampled for them should still be the one at their
original unshifted world position.

Regular cells and voxels are addressed by x,y,z indices

Transition cells, on the other hand, can only appear on external faces of the block.
For addressing them, we do not use the world global XYZ system:
For each of the six faces of the block, a rotated UVW system is defined and used, where UV
are coordinates along the face (from an origin also defined by the face), and W is perpendicular
to the face (with +W toward the inside of the cube). W is used to address voxels only, as cells
faces just need UV to be positionned within the block face.

For cells UV, 1 unit is 1 cell (ex +1U moves to the next cell in the U direction).
Voxels UVW (`HighResolutionVoxelDelta`) start at the UV base of the cell, and 1 unit is half the length of a face. (ex +1U +1V is in the middle of the face)

See [Rotation] for the definitions of UVW for each side of the block.

*/
use std::ops::{Add, Sub};

use crate::traits::Coordinate;

use super::implementation::rotation::Rotation;
use super::mesh_builder::Position;
use super::transition_sides::TransitionSide;
use super::voxel_source::Block;

/// Coordinates of a regular cell within the block. Go from 0 to BLOCK_SIZE - 1
#[derive(Debug, PartialEq)]
pub struct RegularCellIndex {
    /// X. From 0 to `subdivisions` - 1 (included)
    pub x: usize,
    /// Y From 0 to `subdivisions` - 1 (included)
    pub y: usize,
    /// Y From 0 to `subdivisions` - 1 (included)
    pub z: usize,
}

/// XYZ index of a regular voxel relative to the base of a regular cell, or to another regular voxel. 1 unit is 1 cell's size
#[derive(Clone, Copy)]
pub struct RegularVoxelDelta {
    /// X
    pub x: isize,
    /// Y
    pub y: isize,
    /// Y
    pub z: isize,
}

/// Index of a regular voxel relative to a block. It can refer to a voxel outside of the block, as we need to reach farther out to compute normals
#[derive(Debug, PartialEq)]
pub struct RegularVoxelIndex {
    /// X-index. From -1 to `subdivisions` + 1 (included)
    pub x: isize,
    /// Y-index. From -1 to `subdivisions` + 1 (included)
    pub y: isize,
    /// Z-index. From -1 to `subdivisions` + 1 (included)
    pub z: isize,
}

impl Add<&RegularVoxelDelta> for &RegularCellIndex {
    type Output = RegularVoxelIndex;

    fn add(self, rhs: &RegularVoxelDelta) -> Self::Output {
        RegularVoxelIndex {
            x: self.x as isize + rhs.x,
            y: self.y as isize + rhs.y,
            z: self.z as isize + rhs.z,
        }
    }
}

impl Add<RegularVoxelDelta> for &RegularVoxelIndex {
    type Output = RegularVoxelIndex;

    fn add(self, rhs: RegularVoxelDelta) -> Self::Output {
        RegularVoxelIndex {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

/// Index of a transition cell within a block
#[derive(Copy, Clone, Debug)]
pub struct TransitionCellIndex {
    /// The block face on which this cell is
    pub side: TransitionSide,
    /// U-index. From 0 to `subdivisions` - 1 (included)
    pub cell_u: usize,
    /// V-index. From 0 to `subdivisions` - 1 (included)
    pub cell_v: usize,
}

impl TransitionCellIndex {
    /// Shorthand constructor
    pub fn from(side: TransitionSide, cell_u: usize, cell_v: usize) -> Self {
        Self {
            side,
            cell_u,
            cell_v,
        }
    }
}

/// Index of a high resolution voxel within a transition cell
#[derive(Copy, Clone, Debug)]
pub struct HighResolutionVoxelDelta {
    /// U. From -1 to 3 (included). 0 to 2 are within the cell. -1 and 3 extend out, for gradient computations
    pub u: isize,
    /// V. From -1 to 3 (included). 0 to 2 are within the cell. -1 and 3 extend out, for gradient computations
    pub v: isize,
    /// W. From -1 to 1. 0 is on the face, 1 is within the cell, -1 is outside the cell
    pub w: isize,
}

/// Index of a high resolution voxel within a block
#[derive(Debug)]
pub struct HighResolutionVoxelIndex {
    /// Cell within the block
    pub cell: TransitionCellIndex,
    /// Voxel within the cell
    pub delta: HighResolutionVoxelDelta,
}

impl HighResolutionVoxelIndex {
    /// Shorthand constructor
    pub fn from(
        side: TransitionSide,
        cell_u: usize,
        cell_v: usize,
        u: isize,
        v: isize,
        w: isize,
    ) -> Self {
        HighResolutionVoxelIndex {
            cell: TransitionCellIndex::from(side, cell_u, cell_v),
            delta: HighResolutionVoxelDelta::from(u, v, w),
        }
    }

    /// Whether the voxel coincides with a voxel on the "regular" grid
    pub fn on_regular_grid(&self) -> bool {
        let du_on_regular_grid = (self.delta.u % 2) == 0;
        let dv_on_regular_grid = (self.delta.v % 2) == 0;
        let dw_on_regular_grid = self.delta.w == 0;
        du_on_regular_grid && dv_on_regular_grid && dw_on_regular_grid
    }

    /// Convert to the coinciding regular voxel index. Only valid if `self.on_regular_grid()`
    pub fn as_regular_index(
        &self,
        rotation: &Rotation,
        block_subdivisions: usize,
    ) -> RegularVoxelIndex {
        debug_assert!(rotation.side == self.cell.side);
        let cell_u = self.delta.u as usize / 2;
        let cell_v = self.delta.v as usize / 2;
        rotation.to_regular_voxel_index(block_subdivisions, &self.cell, cell_u, cell_v)
    }

    /// Convert to a relative x, y, z position within the block (0,0,0 being at the block origin, 1,1,1 at the opposite max end)
    pub fn to_position_in_block<F>(&self, block: &Block<F>) -> Position<F>
    where
        F: Coordinate,
    {
        let rotation = Rotation::for_side(self.cell.side);
        rotation.to_position_in_block(block.subdivisions, self)
    }

    /// `self` being a double-resolution voxel on a transition face in this block, it coincides with a regular voxel on the neighbouring block at that face. This gives that voxel's index within that block
    pub fn to_higher_res_neighbour_block_index(&self, this_block_size: usize) -> RegularVoxelIndex {
        let higher_res_block_size = this_block_size as isize * 2;
        let cell = self.cell;
        let delta = self.delta;
        let rot = Rotation::for_side(cell.side);
        let x = higher_res_block_size * (rot.uvw_base.x + rot.w.x)
            + delta.w * rot.w.x
            + (2 * cell.cell_u as isize + delta.u) * rot.u.x
            + (2 * cell.cell_v as isize + delta.v) * rot.v.x;
        let y = higher_res_block_size * (rot.uvw_base.y + rot.w.y)
            + delta.w * rot.w.y
            + (2 * cell.cell_u as isize + delta.u) * rot.u.y
            + (2 * cell.cell_v as isize + delta.v) * rot.v.y;
        let z = higher_res_block_size * (rot.uvw_base.z + rot.w.z)
            + delta.w * rot.w.z
            + (2 * cell.cell_u as isize + delta.u) * rot.u.z
            + (2 * cell.cell_v as isize + delta.v) * rot.v.z;
        RegularVoxelIndex { x, y, z }
    }
}

impl HighResolutionVoxelDelta {
    /// Shorthand constructor
    pub const fn from(u: isize, v: isize, w: isize) -> Self {
        Self { u, v, w }
    }
}

impl Add<&HighResolutionVoxelDelta> for &TransitionCellIndex {
    type Output = HighResolutionVoxelIndex;

    fn add(self, rhs: &HighResolutionVoxelDelta) -> Self::Output {
        HighResolutionVoxelIndex {
            cell: *self,
            delta: *rhs,
        }
    }
}

impl Add<&HighResolutionVoxelDelta> for &HighResolutionVoxelIndex {
    type Output = HighResolutionVoxelIndex;

    fn add(self, rhs: &HighResolutionVoxelDelta) -> Self::Output {
        HighResolutionVoxelIndex {
            cell: self.cell,
            delta: HighResolutionVoxelDelta::from(
                self.delta.u + rhs.u,
                self.delta.v + rhs.v,
                self.delta.w + rhs.w,
            ),
        }
    }
}

impl Sub<&HighResolutionVoxelDelta> for &HighResolutionVoxelIndex {
    type Output = HighResolutionVoxelIndex;

    fn sub(self, rhs: &HighResolutionVoxelDelta) -> Self::Output {
        HighResolutionVoxelIndex {
            cell: self.cell,
            delta: HighResolutionVoxelDelta::from(
                self.delta.u - rhs.u,
                self.delta.v - rhs.v,
                self.delta.w - rhs.w,
            ),
        }
    }
}
