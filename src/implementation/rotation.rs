/*!
Mapping system between XYZ and UVW coordinates
*/
use crate::mesh_builder::Position;
use crate::traits::Coordinate;

use super::super::transition_sides::*;
use super::super::voxel_coordinates::*;
use super::aux_tables;

#[derive(Debug)]
pub struct Xyz {
    pub x: isize,
    pub y: isize,
    pub z: isize,
}
#[derive(Debug)]
/**
Rotation information for one of the 6 [TransitionSide]
*/
pub struct Rotation {
    /// The side is just stored along here for convenience
    pub side: TransitionSide,
    /// Where the origin of the UVW system is, in the XYZ system. It is used both for cells and blocks. 0 is at the lowest of the cell or block, 1 at the highest
    pub uvw_base: Xyz,
    /// The direction of the U unit vector, in the XYZ system. Components can be -1 0 or 1
    pub u: Xyz,
    /// The direction of the V unit vector, in the XYZ system. Components can be -1 0 or 1
    pub v: Xyz,
    /// The direction of the W unit vector, in the XYZ system. Components can be -1 0 or 1
    pub w: Xyz,
    /// The direction of the X unit vector, in the UVW system. Components can be -1 0 or 1
    pub plus_x_as_uvw: HighResolutionVoxelDelta,
    /// The direction of the Y unit vector, in the UVW system. Components can be -1 0 or 1
    pub plus_y_as_uvw: HighResolutionVoxelDelta,
    /// The direction of the Z unit vector, in the UVW system. Components can be -1 0 or 1
    pub plus_z_as_uvw: HighResolutionVoxelDelta,
}

impl Xyz {
    const fn from(xyz: (isize, isize, isize)) -> Self {
        Xyz {
            x: xyz.0,
            y: xyz.1,
            z: xyz.2,
        }
    }
}

impl Rotation {
    pub fn for_side(side: TransitionSide) -> &'static Self {
        &aux_tables::ROTATIONS[side as usize]
    }

    /**
    Gives the position of this voxel relative to the block (each coordinate ranging from 0 to 1)
    */
    pub fn to_position_in_block<F>(
        &self,
        block_size: usize,
        voxel_index: &HighResolutionVoxelIndex,
    ) -> Position<F>
    where
        F: Coordinate,
    {
        let cell_index = voxel_index.cell;
        let delta = voxel_index.delta;
        // We multiply by 2 most things to divide in the end, in an attempt to reduce floating point operations (maybe need to measure if this is gaining us anything)
        let x = self.uvw_base.x * 2 * block_size as isize
            + self.u.x * (2 * cell_index.cell_u as isize + delta.u)
            + self.v.x * (2 * cell_index.cell_v as isize + delta.v)
            + self.w.x * delta.w;
        let x = F::half(x) * F::from_ratio(1, block_size);
        let y = self.uvw_base.y * 2 * block_size as isize
            + self.u.y * (2 * cell_index.cell_u as isize + delta.u)
            + self.v.y * (2 * cell_index.cell_v as isize + delta.v)
            + self.w.y * delta.w;
        let y = F::half(y) * F::from_ratio(1, block_size);
        let z = self.uvw_base.z * 2 * block_size as isize
            + self.u.z * (2 * cell_index.cell_u as isize + delta.u)
            + self.v.z * (2 * cell_index.cell_v as isize + delta.v)
            + self.w.z * delta.w;
        let z = F::half(z) * F::from_ratio(1, block_size);
        Position { x, y, z }
    }

    pub fn to_regular_voxel_index(
        &self,
        block_size: usize,
        cell_index: &TransitionCellIndex,
        face_u: usize,
        face_v: usize,
    ) -> RegularVoxelIndex {
        let x = self.uvw_base.x * block_size as isize
            + self.u.x * (cell_index.cell_u + face_u) as isize
            + self.v.x * (cell_index.cell_v + face_v) as isize;
        let y = self.uvw_base.y * block_size as isize
            + self.u.y * (cell_index.cell_u + face_u) as isize
            + self.v.y * (cell_index.cell_v + face_v) as isize;
        let z = self.uvw_base.z * block_size as isize
            + self.u.z * (cell_index.cell_u + face_u) as isize
            + self.v.z * (cell_index.cell_v + face_v) as isize;
        RegularVoxelIndex { x, y, z }
    }

    #[allow(clippy::too_many_arguments)]
    pub const fn from(
        side: TransitionSide,
        uvw_base: (isize, isize, isize),
        u: (isize, isize, isize),
        v: (isize, isize, isize),
        w: (isize, isize, isize),
        _xyz_base: (isize, isize, isize),
        x: (isize, isize, isize),
        y: (isize, isize, isize),
        z: (isize, isize, isize),
    ) -> Self {
        Rotation {
            side,
            uvw_base: Xyz::from(uvw_base),
            u: Xyz::from(u),
            v: Xyz::from(v),
            w: Xyz::from(w),
            plus_x_as_uvw: HighResolutionVoxelDelta::from(x.0, x.1, x.2),
            plus_y_as_uvw: HighResolutionVoxelDelta::from(y.0, y.1, y.2),
            plus_z_as_uvw: HighResolutionVoxelDelta::from(z.0, z.1, z.2),
        }
    }

    pub fn default() -> &'static Self {
        &aux_tables::ROTATIONS[0]
    }
}
