/*!
Trait for accessing world data
*/

use crate::traits::{coordinate::Coordinate, voxel_data::VoxelData};

/**
A source of "world" voxel data (gives data for any world x,y,z coordinates)
*/
pub trait DataField<V: VoxelData, C: Coordinate> {
    /**
    Obtain the data at the given point in space
    */
    fn get_data(&self, x: C, y: C, z: C) -> V;
}

/**
DataField implementation for references
*/
impl<V: VoxelData, C: Coordinate> DataField<V, C> for &dyn DataField<V, C> {
    fn get_data(&self, x: C, y: C, z: C) -> V {
        (*self).get_data(x, y, z)
    }
}

/**
DataField implementation for closures
 */
impl<V: VoxelData, C: Coordinate, FN> DataField<V, C> for FN
where
    FN: Fn(C, C, C) -> V,
{
    fn get_data(&self, x: C, y: C, z: C) -> V {
        self(x, y, z)
    }
}
