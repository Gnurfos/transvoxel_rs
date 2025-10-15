/*!
Grid point struct
*/

use std::fmt::Debug;


use crate::structs::position::OutputPosition;
use crate::traits::coordinate::Coordinate;
use crate::traits::voxel_data::VoxelData;

/// A grid point on the voxel grid, but expressed in world coordinates (floats).
/// Also stores evaluated voxel data and density gradient at that point.
/// A pair of these will be passed to the mesh generator, when creating vertices
#[derive(Debug)]
pub struct GridPoint<V: VoxelData, C: Coordinate> {
    /// World location of the grid point
    pub position: OutputPosition<C>,
    /// Density gradient (estimated) at the grid point
    pub gradient: (V::Density, V::Density, V::Density),
    /// Data at the grid point that was obtained from the field
    pub voxel_data: V,
}
