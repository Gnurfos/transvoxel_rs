use super::super::structs::*;
use super::super::transition_sides::*;
use super::super::voxel_coordinates::*;
use super::aux_tables;

#[derive(Debug)]
pub struct XYZ {
    pub x: isize,
    pub y: isize,
    pub z: isize,
}
#[derive(Debug)]
pub struct Rotation {
    pub side: TransitionSide,
    pub uvw_base: XYZ,
    pub u: XYZ,
    pub v: XYZ,
    pub w: XYZ,
    pub plus_x_as_uvw: HighResolutionVoxelDelta,
    pub plus_y_as_uvw: HighResolutionVoxelDelta,
    pub plus_z_as_uvw: HighResolutionVoxelDelta,
}

impl XYZ {
    const fn from(xyz: (isize, isize, isize)) -> Self {
        XYZ {
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

    pub fn to_position_in_block(
        &self,
        block_size: usize,
        voxel_index: &HighResolutionVoxelIndex,
    ) -> Position {
        let cell_index = voxel_index.cell;
        let delta = voxel_index.delta;
        // We multiply by 2 most things to divide in the end, in an attempt to reduce floating point operations (maybe need to measure if this is gaining us anything)
        let x = self.uvw_base.x * 2 * block_size as isize
            + self.u.x * (2 * cell_index.cell_u as isize + delta.u)
            + self.v.x * (2 * cell_index.cell_v as isize + delta.v)
            + self.w.x * delta.w;
        let x = x as f32 * 0.5f32;
        let y = self.uvw_base.y * 2 * block_size as isize
            + self.u.y * (2 * cell_index.cell_u as isize + delta.u)
            + self.v.y * (2 * cell_index.cell_v as isize + delta.v)
            + self.w.y * delta.w;
        let y = y as f32 * 0.5f32;
        let z = self.uvw_base.z * 2 * block_size as isize
            + self.u.z * (2 * cell_index.cell_u as isize + delta.u)
            + self.v.z * (2 * cell_index.cell_v as isize + delta.v)
            + self.w.z * delta.w;
        let z = z as f32 * 0.5f32;
        Position { x: x, y: y, z: z }
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
        return RegularVoxelIndex { x, y, z };
    }

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
            side: side,
            uvw_base: XYZ::from(uvw_base),
            u: XYZ::from(u),
            v: XYZ::from(v),
            w: XYZ::from(w),
            plus_x_as_uvw: HighResolutionVoxelDelta::from(x.0, x.1, x.2),
            plus_y_as_uvw: HighResolutionVoxelDelta::from(y.0, y.1, y.2),
            plus_z_as_uvw: HighResolutionVoxelDelta::from(z.0, z.1, z.2),
        }
    }

    pub fn default() -> &'static Self {
        &aux_tables::ROTATIONS[0]
    }
}
