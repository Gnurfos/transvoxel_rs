/*!
Traits/structs for providing access to a voxel grid
*/

use super::{
    density::{Density, ScalarField, Coordinate},
    structs::Block,
    voxel_coordinates::{HighResolutionVoxelIndex, RegularVoxelIndex},
};

/**
Wrapper used to retrieve density values for a [Block] (typically from an underlying [ScalarField])
This source is accessed through [voxel_coordinates] structs

[voxel_coordinates]: super::voxel_coordinates
*/
pub trait VoxelSource<D: Density> {
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
    fn get_density(&self, voxel_index: &RegularVoxelIndex) -> D;

    /**
    If the extraction needs to handle some transition faces, this method will be called to get density for non regular voxels
    It will only get called for non-regular voxels (ie odd u/v and/or non-zero w): For example for U=2,V=0,W=0 the algorithm
    will try to call `get_density` instead
    */
    fn get_transition_density(&self, index: &HighResolutionVoxelIndex) -> D;
}

/**
Relays calls to the underlying field every time density is queried
Maps voxel coordinates to world x,y,z coordinates (using the block context)
The most interesting logic lies in converting UVW coordinates relative to one transition side, to XYZ coordinates

The underlying field can be owned, or just a reference, as a reference to a [ScalarField] is also a [ScalarField]
*/
pub struct WorldMappingVoxelSource<'b, S, C>
where C: Coordinate,
{
    /// [ScalarField] used to access world densities
    pub field: S,
    /// [Block] context, used to mapping coordinates
    pub block: &'b Block<C>,
}

impl<'a, 'b, S, D, C> VoxelSource<D> for WorldMappingVoxelSource<'b, S, C>
where
    D: Density,
    S: ScalarField<D, C>,
    C: Coordinate,
{
    fn get_density(&self, voxel_index: &RegularVoxelIndex) -> D {
        let x = self.block.dims.base[0]
            + self.block.dims.size * C::from_ratio(voxel_index.x, self.block.subdivisions);
        let y = self.block.dims.base[1]
            + self.block.dims.size * C::from_ratio(voxel_index.y, self.block.subdivisions);
        let z = self.block.dims.base[2]
            + self.block.dims.size * C::from_ratio(voxel_index.z, self.block.subdivisions);
        let d = self.field.get_density(x, y, z);
        d
    }

    fn get_transition_density(&self, index: &HighResolutionVoxelIndex) -> D {
        let rotation = super::implementation::rotation::Rotation::for_side(index.cell.side);
        let position_in_block = rotation.to_position_in_block::<C>(self.block.subdivisions, index);
        let x = self.block.dims.base[0]
            + self.block.dims.size * position_in_block.x;
        let y = self.block.dims.base[1]
            + self.block.dims.size * position_in_block.y;
        let z = self.block.dims.base[2]
            + self.block.dims.size * position_in_block.z;
        self.field.get_density(x, y, z)
    }
}

/**
VoxelSource implementation for references
*/

impl<D, F> VoxelSource<D> for &mut F
where
    D: Density,
    F: VoxelSource<D> + ?Sized,
{
    fn get_density(&self, voxel_index: &RegularVoxelIndex) -> D {
        (**self).get_density(voxel_index)
    }

    fn get_transition_density(&self, index: &HighResolutionVoxelIndex) -> D {
        (**self).get_transition_density(index)
    }
}
