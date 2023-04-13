use std::collections::HashMap;

use super::super::{
    density::Density,
    transition_sides::TransitionSides,
    voxel_coordinates::{HighResolutionVoxelIndex, RegularVoxelIndex},
    voxel_source::VoxelSource,
};

pub struct PreCachingVoxelSource<D, S> {
    inner_source: S,
    block_subdivisions: usize,
    regular_cache: Vec<D>,
    regular_cache_extended: Vec<D>,
    regular_cache_extended_loaded: bool,
    transition_cache: Vec<D>,
    transition_cache_loaded: bool,
    transition_cache_slices: HashMap<usize, usize>, // side -> slice in the cache
}

impl<D, S> PreCachingVoxelSource<D, S>
where
    D: Density,
    S: VoxelSource<D>,
{
    pub fn new(source: S, block_subdivisions: usize) -> Self {
        let mut object = Self {
            inner_source: source,
            block_subdivisions,
            regular_cache: Vec::new(),
            regular_cache_extended: Vec::new(),
            regular_cache_extended_loaded: false,
            transition_cache: Vec::new(),
            transition_cache_loaded: false,
            transition_cache_slices: HashMap::new(),
        };
        object.load_regular_block_voxels();
        object
    }

    fn load_regular_block_voxels(&mut self) {
        let subs = self.block_subdivisions;
        self.regular_cache
            .resize((subs + 1) * (subs + 1) * (subs + 1), D::default());
        for x in 0..=subs {
            for y in 0..=subs {
                for z in 0..=subs {
                    let index = self.regular_block_index(x as isize, y as isize, z as isize);
                    self.regular_cache[index] =
                        self.from_source(x as isize, y as isize, z as isize);
                }
            }
        }
    }

    fn regular_block_index(&self, x: isize, y: isize, z: isize) -> usize {
        let subs = self.block_subdivisions;
        (subs + 1) * (subs + 1) * x as usize + (subs + 1) * y as usize + z as usize
    }

    // These are the regular-spaced voxels but outside of the block
    // We can load them lazily using this function only when we know some vertices will come out of the block and we need gradients/normals for them
    // At the moment, the control does not extend to differenciating between "some vertices" and "some vertices actually on the edge of the block"
    pub fn load_regular_extended_voxels(&mut self) {
        if self.regular_cache_extended_loaded {
            return;
        } else {
            self.regular_cache_extended_loaded = true;
        }
        let subs = self.block_subdivisions;
        let face_size = (subs + 1) * (subs + 1);
        self.regular_cache_extended.resize(6 * face_size, D::default());
        // -x
        for y in 0..=subs {
            for z in 0..=subs {
                let index = 0 * face_size + (subs + 1) * y as usize + z as usize;
                self.regular_cache_extended[index] = self.from_source(-1, y as isize, z as isize);
            }
        }
        // +x
        for y in 0..=subs {
            for z in 0..=subs {
                let index = 1 * face_size + (subs + 1) * y as usize + z as usize;
                self.regular_cache_extended[index] =
                    self.from_source(subs as isize + 1, y as isize, z as isize);
            }
        }
        // -y
        for x in 0..=subs {
            for z in 0..=subs {
                let index = 2 * face_size + (subs + 1) * x as usize + z as usize;
                self.regular_cache_extended[index] = self.from_source(x as isize, -1, z as isize);
            }
        }
        // +y
        for x in 0..=subs {
            for z in 0..=subs {
                let index = 3 * face_size + (subs + 1) * x as usize + z as usize;
                self.regular_cache_extended[index] =
                    self.from_source(x as isize, subs as isize + 1, z as isize);
            }
        }
        // -z
        for x in 0..=subs {
            for y in 0..=subs {
                let index = 4 * face_size + (subs + 1) * x as usize + y as usize;
                self.regular_cache_extended[index] = self.from_source(x as isize, y as isize, -1);
            }
        }
        // +z
        for x in 0..=subs {
            for y in 0..=subs {
                let index = 5 * face_size + (subs + 1) * x as usize + y as usize;
                self.regular_cache_extended[index] =
                    self.from_source(x as isize, y as isize, subs as isize + 1);
            }
        }
    }

    pub fn load_transition_voxels(&mut self, transition_sides: TransitionSides) {
        if self.transition_cache_loaded {
            return;
        } else {
            self.transition_cache_loaded = true;
        }
        let mut num_transitions = 0usize;
        for side in transition_sides {
            self.transition_cache_slices
                .insert(side as usize, num_transitions);
            num_transitions += 1;
        }
        // We will only store the w=0 voxels, and not the ones out of the block (so, all voxels for case computations, and vertex positions, but not for gradients)
        // For simplicity (at the cost of compactness) we store a sparse array also containing regular voxels on the face, that will never get read/written
        let subs = self.block_subdivisions;
        let size_per_face = (2 * subs + 1) * (2 * subs + 1);
        self.transition_cache
            .resize(num_transitions * size_per_face, D::default());
        for side in transition_sides {
            for cell_u in 0..subs {
                for cell_v in 0..subs {
                    self.cache_transition_voxel(&HighResolutionVoxelIndex::from(
                        side, cell_u, cell_v, 1, 0, 0,
                    ));
                    self.cache_transition_voxel(&HighResolutionVoxelIndex::from(
                        side, cell_u, cell_v, 0, 1, 0,
                    ));
                    self.cache_transition_voxel(&HighResolutionVoxelIndex::from(
                        side, cell_u, cell_v, 1, 1, 0,
                    ));
                }
                self.cache_transition_voxel(&HighResolutionVoxelIndex::from(
                    side,
                    cell_u,
                    subs - 1,
                    1,
                    2,
                    0,
                ));
            }
            for cell_v in 0..subs {
                self.cache_transition_voxel(&HighResolutionVoxelIndex::from(
                    side,
                    subs - 1,
                    cell_v,
                    2,
                    1,
                    0,
                ));
            }
        }
    }

    fn cache_transition_voxel(&mut self, voxel_index: &HighResolutionVoxelIndex) {
        let d = self.inner_source.get_transition_density(voxel_index);
        let cache_index = self.transition_cache_index(voxel_index);
        self.transition_cache[cache_index] = d;
    }

    fn transition_cache_index(&self, voxel_index: &HighResolutionVoxelIndex) -> usize {
        let side = voxel_index.cell.side as usize;
        let cache_slice = self.transition_cache_slices.get(&side).unwrap();
        let subs = self.block_subdivisions;
        let size_per_face = (2 * subs + 1) * (2 * subs + 1);
        let slice_shift = cache_slice * size_per_face;
        let global_du = 2 * voxel_index.cell.cell_u as isize + voxel_index.delta.u;
        let global_dv = 2 * voxel_index.cell.cell_v as isize + voxel_index.delta.v;
        let index_in_slice = (2 * subs + 1) * global_du as usize + global_dv as usize;
        slice_shift + index_in_slice
    }

    fn from_source(&mut self, x: isize, y: isize, z: isize) -> D {
        self.inner_source
            .get_density(&RegularVoxelIndex { x, y, z })
    }
}

impl<D, S> PreCachingVoxelSource<D, S>
where
    D: Density,
    S: VoxelSource<D>,
{
    pub fn get_density(&mut self, voxel_index: &RegularVoxelIndex) -> D {
        let x = voxel_index.x;
        let y = voxel_index.y;
        let z = voxel_index.z;
        let subs = self.block_subdivisions;
        let face_size = (subs + 1) * (subs + 1);
        if x == -1 {
            debug_assert!(y >= 0 && y <= subs as isize && z >= 0 && z <= subs as isize);
            let index = 0 * face_size + (subs + 1) * y as usize + z as usize;
            self.load_regular_extended_voxels();
            return self.regular_cache_extended[index];
        } else if x == subs as isize + 1 {
            debug_assert!(y >= 0 && y <= subs as isize && z >= 0 && z <= subs as isize);
            let index = 1 * face_size + (subs + 1) * y as usize + z as usize;
            self.load_regular_extended_voxels();
            return self.regular_cache_extended[index];
        } else if y == -1 {
            debug_assert!(x >= 0 && x <= subs as isize && z >= 0 && z <= subs as isize);
            let index = 2 * face_size + (subs + 1) * x as usize + z as usize;
            self.load_regular_extended_voxels();
            return self.regular_cache_extended[index];
        } else if y == subs as isize + 1 {
            debug_assert!(x >= 0 && x <= subs as isize && z >= 0 && z <= subs as isize);
            let index = 3 * face_size + (subs + 1) * x as usize + z as usize;
            self.load_regular_extended_voxels();
            return self.regular_cache_extended[index];
        } else if z == -1 {
            debug_assert!(x >= 0 && x <= subs as isize && y >= 0 && y <= subs as isize);
            let index = 4 * face_size + (subs + 1) * x as usize + y as usize;
            self.load_regular_extended_voxels();
            return self.regular_cache_extended[index];
        } else if z == subs as isize + 1 {
            debug_assert!(x >= 0 && x <= subs as isize && y >= 0 && y <= subs as isize);
            let index = 5 * face_size + (subs + 1) * x as usize + y as usize;
            self.load_regular_extended_voxels();
            return self.regular_cache_extended[index];
        } else {
            debug_assert!(
                x >= 0
                    && x <= subs as isize
                    && y >= 0
                    && y <= subs as isize
                    && z >= 0
                    && z <= subs as isize
            );
            let index = self.regular_block_index(x, y, z);
            return self.regular_cache[index];
        }
    }

    pub fn get_transition_density(&self, index: &HighResolutionVoxelIndex) -> D {
        let c = index.cell;
        let d = index.delta;
        let subs = self.block_subdivisions as isize;
        debug_assert!(d.w != 0 || d.u % 2 != 0 || d.v % 2 != 0);
        // The following check is only valid if, for voxels coinciding with a regular voxel, we also get the gradient from the regular voxel. Not sure this is correct (see `high_res_face_grid_point_gradient`)
        if d.w != 0 {
            debug_assert!(d.u % 2 != 0 || d.v % 2 != 0);
        }
        if (d.w != 0)
            || (c.cell_u as isize * 2 + d.u < 0)
            || (c.cell_u as isize * 2 + d.u > 2 * subs)
            || (c.cell_v as isize * 2 + d.v < 0)
            || (c.cell_v as isize * 2 + d.v > 2 * subs)
        {
            // Out of the block face: we don't cache these
            return self.inner_source.get_transition_density(index);
        }
        let cache_index = self.transition_cache_index(index);
        return self.transition_cache[cache_index];
    }
}
