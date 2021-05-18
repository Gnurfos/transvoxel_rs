use super::super::transition_sides::TransitionSide::*;
use super::super::voxel_coordinates::*;
use super::rotation::*;

// Indexed by TransitionSide
// Gives world coords of UVW: base, XYZ of U, XYZ of V, XYZ of W
// and UVW coords of world coords: base, UVW of X, UVW of Y, UVW of Z
#[rustfmt::skip]
pub const ROTATIONS: [Rotation; 6] = [
    Rotation::from(
        LowX,
        (0, 0, 1), (0, 0, -1), (0, 1, 0), (1, 0, 0),
        (1, 0, 0), (0, 0, 1), (0, 1, 0), (-1, 0, 0)), // +X is +W, +Y is +V, +Z is -U
    Rotation::from(
        HighX,
        (1, 0, 0), (0, 0, 1), (0, 1, 0), (-1, 0, 0),
        (0, 0, 1), (0, 0, -1), (0, 1, 0), (1, 0, 0)),
    Rotation::from(
        LowY,
        (0, 0, 1), (1, 0, 0), (0, 0, -1), (0, 1, 0), // U: +X, V: -Z, W: +Y
        (0, 1, 0), (1, 0, 0), (0, 0, 1), (0, -1, 0)),
    Rotation::from(
        HighY,
        (0, 1, 0), (1, 0, 0), (0, 0, 1), (0, -1, 0),
        (0, 0, 1), (1, 0, 0), (0, 0, -1), (0, 1, 0)),
    Rotation::from(
        LowZ,
        (0, 0, 0), (1, 0, 0), (0, 1, 0), (0, 0, 1), // U: +X, V: +Y, W: +Z
        (0, 0, 0), (1, 0, 0), (0, 1, 0), (0, 0, 1)),
    Rotation::from(
        HighZ,
        (1, 0, 1), (-1, 0, 0), (0, 1, 0), (0, 0, -1),
        (1, 0, 1), (-1, 0, 0), (0, 1, 0), (0, 0, -1)),
];

pub struct RegularCellVoxelIndex(pub usize);

pub const REGULAR_CELL_VOXELS: [RegularVoxelDelta; 8] = [
    RegularVoxelDelta { x: 0, y: 0, z: 0 }, // Voxel 0 is the cell "origin" [with the lowest x, y, and z]
    RegularVoxelDelta { x: 1, y: 0, z: 0 }, // Voxel 1 == 1 toward X
    RegularVoxelDelta { x: 0, y: 1, z: 0 }, // Voxel 2 == 1 toward Y
    RegularVoxelDelta { x: 1, y: 1, z: 0 },
    RegularVoxelDelta { x: 0, y: 0, z: 1 },
    RegularVoxelDelta { x: 1, y: 0, z: 1 },
    RegularVoxelDelta { x: 0, y: 1, z: 1 },
    RegularVoxelDelta { x: 1, y: 1, z: 1 },
];

pub fn get_regular_voxel_delta(index: RegularCellVoxelIndex) -> RegularVoxelDelta {
    REGULAR_CELL_VOXELS[index.0]
}

// From 0 to C
pub struct TransitionCellGridPointIndex(pub usize);

pub enum TransitionCellGridPoint {
    HighResFace(HighResolutionVoxelDelta),
    RegularFace(usize, usize),
}

const fn tcell_highres_face_gridpoint(u: isize, v: isize) -> TransitionCellGridPoint {
    TransitionCellGridPoint::HighResFace(HighResolutionVoxelDelta { u, v, w: 0 })
}

const fn tcell_reg_face_gridpoint(u: usize, v: usize) -> TransitionCellGridPoint {
    TransitionCellGridPoint::RegularFace(u, v)
}

pub const TRANSITION_CELL_GRID_POINTS: [TransitionCellGridPoint; 13] = [
    tcell_highres_face_gridpoint(0, 0),
    tcell_highres_face_gridpoint(1, 0),
    tcell_highres_face_gridpoint(2, 0),
    tcell_highres_face_gridpoint(0, 1),
    tcell_highres_face_gridpoint(1, 1),
    tcell_highres_face_gridpoint(2, 1),
    tcell_highres_face_gridpoint(0, 2),
    tcell_highres_face_gridpoint(1, 2),
    tcell_highres_face_gridpoint(2, 2),
    tcell_reg_face_gridpoint(0, 0),
    tcell_reg_face_gridpoint(1, 0),
    tcell_reg_face_gridpoint(0, 1),
    tcell_reg_face_gridpoint(1, 1),
];

pub const TRANSITION_HIGH_RES_FACE_CASE_CONTRIBUTIONS: [(HighResolutionVoxelDelta, usize); 9] = [
    (HighResolutionVoxelDelta::from(0, 0, 0), 0x01),
    (HighResolutionVoxelDelta::from(1, 0, 0), 0x02),
    (HighResolutionVoxelDelta::from(2, 0, 0), 0x04),
    (HighResolutionVoxelDelta::from(0, 1, 0), 0x80),
    (HighResolutionVoxelDelta::from(1, 1, 0), 0x100),
    (HighResolutionVoxelDelta::from(2, 1, 0), 0x08),
    (HighResolutionVoxelDelta::from(0, 2, 0), 0x40),
    (HighResolutionVoxelDelta::from(1, 2, 0), 0x20),
    (HighResolutionVoxelDelta::from(2, 2, 0), 0x10),
];
