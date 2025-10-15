#![allow(missing_docs)]
/*!
Implementations of various [VoxelBlock]
*/

use std::mem::{transmute, MaybeUninit};

use crate::structs::block::Block;
use crate::structs::position::SamplingPosition;
use crate::structs::voxel_index::VoxelIndex;
use crate::traits::data_field::DataField;
use crate::traits::voxel_block::VoxelBlock;
use crate::traits::{coordinate::Coordinate, voxel_data::VoxelData};

/// Data for a block + extensions, stored as a single vector
pub struct VoxelVecBlock<C: Coordinate, V: VoxelData + Copy> {
    pub block: Block<C>,
    pub data: Vec<V>, // Layout: voxels (subdivisions + 1)^3 voxels within the block first, then 6x an extension face of (subdivisions + 1)^2
}

impl<C: Coordinate, V: VoxelData + Copy> VoxelBlock<C, V> for VoxelVecBlock<C, V> {
    fn block(&self) -> &Block<C> {
        &self.block
    }

    fn get(&self, index: VoxelIndex) -> V {
        self.data[self.storage_index(index)]
    }
}

impl<C: Coordinate, V: VoxelData + Copy> VoxelVecBlock<C, V> {
    fn cache_one<F: DataField<V, C> + ?Sized>(
        field: &F,
        block: &Block<C>,
        x: isize,
        y: isize,
        z: isize,
        data: &mut [MaybeUninit<V>],
    ) {
        let subdivisions = block.subdivisions;
        let voxel_index = VoxelIndex { x, y, z };
        let storage_index = Self::storage_index_aux(subdivisions, voxel_index);
        let SamplingPosition { x, y, z } = block.original_voxel_position(voxel_index);
        let v = field.get_data(x, y, z);
        data[storage_index].write(v);
    }

    /// Evaluate the field and store its value at every point that might be queried
    pub fn cache<F: DataField<V, C> + ?Sized>(
        field: &F,
        block: Block<C>,
    ) -> Self {
        let subs = block.subdivisions;
        let size = (subs + 1) * (subs + 1) * (subs + 1) + 6 * (subs + 1) * (subs + 1);
        let mut data = Vec::<MaybeUninit<V>>::with_capacity(size);
        unsafe { data.set_len(size); }
        for x in 0..=subs {
            for y in 0..=subs {
                for z in 0..=subs {
                    Self::cache_one(field, &block, x as isize, y as isize, z as isize, &mut data);
                }
            }
        }
        for i in 0..=subs {
            for j in 0..=subs {
                Self::cache_one(field, &block, -1, i as isize, j as isize, &mut data);
                Self::cache_one(field, &block, subs as isize + 1, i as isize, j as isize, &mut data);
                Self::cache_one(field, &block, i as isize, -1, j as isize, &mut data);
                Self::cache_one(field, &block, i as isize, subs as isize + 1, j as isize, &mut data);
                Self::cache_one(field, &block, i as isize, j as isize, -1, &mut data);
                Self::cache_one(field, &block, i as isize, j as isize, subs as isize+ 1, &mut data);
            }
        }
        let data = unsafe {
            transmute::<Vec<MaybeUninit<V>>, Vec<V>>(data)
        };
        Self {
            block,
            data,
        }
    }
}

impl<C: Coordinate, V: VoxelData + Copy> VoxelVecBlock<C, V> {
    fn storage_index(&self, index: VoxelIndex) -> usize {
        Self::storage_index_aux(self.block.subdivisions, index)
    }

    fn storage_index_aux(subs: usize, VoxelIndex {x, y, z}: VoxelIndex) -> usize {
        let voxels_within_block = (subs + 1) * (subs + 1) * (subs + 1);
        let extension_size = (subs + 1) * (subs + 1);
        if x == -1 {
            debug_assert!(y >= 0 && y <= subs as isize && z >= 0 && z <= subs as isize);
            let index_in_face_extension = (subs + 1) * y as usize + z as usize;
            let face_extension_offset = voxels_within_block;
            index_in_face_extension + face_extension_offset
        } else if x == subs as isize + 1 {
            debug_assert!(y >= 0 && y <= subs as isize && z >= 0 && z <= subs as isize);
            let index_in_face_extension = (subs + 1) * y as usize + z as usize;
            let face_extension_offset = voxels_within_block + extension_size;
            index_in_face_extension + face_extension_offset
        } else if y == -1 {
            debug_assert!(x >= 0 && x <= subs as isize && z >= 0 && z <= subs as isize);
            let index_in_face_extension = (subs + 1) * x as usize + z as usize;
            let face_extension_offset = voxels_within_block + 2 * extension_size;
            index_in_face_extension + face_extension_offset
        } else if y == subs as isize + 1 {
            debug_assert!(x >= 0 && x <= subs as isize && z >= 0 && z <= subs as isize);
            let index_in_face_extension = (subs + 1) * x as usize + z as usize;
            let face_extension_offset = voxels_within_block + 3 * extension_size;
            index_in_face_extension + face_extension_offset
        } else if z == -1 {
            debug_assert!(x >= 0 && x <= subs as isize && y >= 0 && y <= subs as isize);
            let index_in_face_extension = (subs + 1) * x as usize + y as usize;
            let face_extension_offset = voxels_within_block + 4 * extension_size;
            index_in_face_extension + face_extension_offset
        } else if z == subs as isize + 1 {
            debug_assert!(x >= 0 && x <= subs as isize && y >= 0 && y <= subs as isize);
            let index_in_face_extension = (subs + 1) * x as usize + y as usize;
            let face_extension_offset = voxels_within_block + 5 * extension_size;
            index_in_face_extension + face_extension_offset
        } else {
            debug_assert!(
                x >= 0
                    && x <= subs as isize
                    && y >= 0
                    && y <= subs as isize
                    && z >= 0
                    && z <= subs as isize
            );
            (subs + 1) * (subs + 1) * x as usize + (subs + 1) * y as usize + z as usize
        }
    }
}



/// A [VoxelBlock] that doesn't store anything, and forwards every query to the underlying field
pub struct VoxelBlockRelayingToField<'a, C: Coordinate, V: VoxelData> {
    pub field: &'a dyn DataField<V, C>,
    pub block: Block<C>,
}

impl<
    C: Coordinate,
    V: VoxelData,
> VoxelBlock<C, V> for VoxelBlockRelayingToField<'_, C, V> {
    fn block(&self) -> &Block<C> {
        &self.block
    }

    fn get(&self, voxel_index: VoxelIndex) -> V {
        let SamplingPosition {x, y, z} = self.block.original_voxel_position(voxel_index);
        self.field.get_data(x, y, z)
    }
}