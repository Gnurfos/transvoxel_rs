/*!
Structs for addressing discrete voxels
 If you subdivide the world or a volume with a grid, "voxels" are the intersection points of the grids
 The cubes between them are called "cells"

*/


/// Index of a voxel relative to a block. It can refer to a voxel slightly outside the block, as we need to reach farther out to compute normals
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct VoxelIndex {
    /// X-index. From -1 to `subdivisions` + 1 (included)
    pub x: isize,
    /// Y-index. From -1 to `subdivisions` + 1 (included)
    pub y: isize,
    /// Z-index. From -1 to `subdivisions` + 1 (included)
    pub z: isize,
}
