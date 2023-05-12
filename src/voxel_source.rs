/*!
Traits/structs for providing access to a voxel grid
*/

use crate::{
    traits::{Coordinate, VoxelData},
    voxel_coordinates::{HighResolutionVoxelIndex, RegularVoxelIndex},
};

/**
A [Block] (cubic region of the world) with attached number of subdivisions for the extraction.
With n subdivisions, the block will contain n cells, encompassing n + 1 voxels across each dimension.
```
# use transvoxel::voxel_source::*;
// Just meant to be constructed and passed around
let a_block = Block {
    dims: BlockDims {
        base: [10.0, 20.0, 30.0],
        size: 10.0,
    },
    subdivisions: 8,
};
let another_block = Block::from([10.0, 20.0, 30.0], 10.0, 8);
```
*/
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Block<C>
where
    C: Coordinate,
{
    /// The zone
    pub dims: BlockDims<C>,
    /// How many subdivisions
    pub subdivisions: usize,
}

impl<C> Block<C>
where
    C: Coordinate,
{
    /// Shortcut constructor
    pub fn from(base: [C; 3], size: C, subdivisions: usize) -> Self {
        Block {
            dims: BlockDims { base, size },
            subdivisions,
        }
    }
}

/**
A cubic zone of the world, for which to run an extraction
*/
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BlockDims<C>
where
    C: Coordinate,
{
    /// Lowest x,y,z point
    pub base: [C; 3],
    /// Side of the cube
    pub size: C,
}

/**
Wrapper used to retrieve voxel values for a [Block] (for example from an underlying [DataField])
This source is accessed through [voxel_coordinates] structs

[voxel_coordinates]: super::voxel_coordinates
*/
pub trait VoxelSource<V: VoxelData> {
    /**
     * This method will be called by the extraction algorithm, for regular voxels
     *
     * It should return the density, at the "base" (lowest corner) of the cell identified by the given voxel index
     *
     * Note that, when computing gradients, this may be queried for cells outside of the extracted block:
     *  - an index of 0 indicates the base of the block
     *  - an index of `subdivisions` indicates the end of the block
     *  - the function can be called for index values between -1 and `subdivisions` + 1 (both included)
     *
     */
    fn get_regular_voxel(&mut self, voxel_index: &RegularVoxelIndex) -> V;

    /**
    If the extraction needs to handle some transition faces, this method will be called to get density for non regular voxels
    It will only get called for non-regular voxels (ie odd u/v and/or non-zero w): For example for U=2,V=0,W=0 the algorithm
    will try to call `get_density` instead
    */
    fn get_transition_voxel(&mut self, index: &HighResolutionVoxelIndex) -> V;
}

/**
Relays calls to the underlying field every time voxel data is queried
Maps voxel coordinates to world x,y,z coordinates (using the block context)
The most interesting logic lies in converting UVW coordinates relative to one transition side, to XYZ coordinates

The underlying field can be owned, or just a reference, as a reference to a [DataField] is also a [DataField]
*/
pub struct WorldMappingVoxelSource<'b, S, C>
where
    C: Coordinate,
{
    /// [DataField] used to access world data
    pub field: S,
    /// [Block] context, used to map coordinates
    pub block: &'b Block<C>,
}

impl<'b, S, V, C> VoxelSource<V> for WorldMappingVoxelSource<'b, S, C>
where
    S: DataField<V, C>,
    C: Coordinate,
    V: VoxelData,
{
    fn get_regular_voxel(&mut self, voxel_index: &RegularVoxelIndex) -> V {
        let x = self.block.dims.base[0]
            + self.block.dims.size * C::from_ratio(voxel_index.x, self.block.subdivisions);
        let y = self.block.dims.base[1]
            + self.block.dims.size * C::from_ratio(voxel_index.y, self.block.subdivisions);
        let z = self.block.dims.base[2]
            + self.block.dims.size * C::from_ratio(voxel_index.z, self.block.subdivisions);
        self.field.get_data(x, y, z)
    }

    fn get_transition_voxel(&mut self, index: &HighResolutionVoxelIndex) -> V {
        let rotation = super::implementation::rotation::Rotation::for_side(index.cell.side);
        let position_in_block = rotation.to_position_in_block::<C>(self.block.subdivisions, index);
        let x = self.block.dims.base[0] + self.block.dims.size * position_in_block.x;
        let y = self.block.dims.base[1] + self.block.dims.size * position_in_block.y;
        let z = self.block.dims.base[2] + self.block.dims.size * position_in_block.z;
        self.field.get_data(x, y, z)
    }
}

/**
VoxelSource implementation for references
*/

impl<V, F> VoxelSource<V> for &mut F
where
    V: VoxelData,
    F: VoxelSource<V> + ?Sized,
{
    fn get_regular_voxel(&mut self, voxel_index: &RegularVoxelIndex) -> V {
        (**self).get_regular_voxel(voxel_index)
    }

    fn get_transition_voxel(&mut self, index: &HighResolutionVoxelIndex) -> V {
        (**self).get_transition_voxel(index)
    }
}

/**
A source of "world" voxel data (gives data for any world x,y,z coordinates)
*/
pub trait DataField<V: VoxelData, C: Coordinate> {
    /**
    Obtain the data at the given point in space
    */
    fn get_data(&mut self, x: C, y: C, z: C) -> V;
}

/**
DataField implementation for references
*/
impl<V: VoxelData, C: Coordinate> DataField<V, C> for &mut dyn DataField<V, C> {
    fn get_data(&mut self, x: C, y: C, z: C) -> V {
        (*self).get_data(x, y, z)
    }
}

/**
DataField implementation for closures
 */
impl<V: VoxelData, C: Coordinate, FN> DataField<V, C> for FN
where
    FN: FnMut(C, C, C) -> V,
{
    fn get_data(&mut self, x: C, y: C, z: C) -> V {
        self(x, y, z)
    }
}
