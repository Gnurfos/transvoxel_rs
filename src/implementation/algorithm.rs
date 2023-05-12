/*!
This is the main algorithm implementation.

At it's heart it's simply:
 1 - for each regular cell, extract its geometry with marching cubes
 2 - for each transition face in the request, extract each transition cell of the face
Both extractions are similar:
 - get densities for all the 8 or 9 voxels of the cell
 - compute the case and class, from densities
 - get triangulation info from the case/class
 - output vertices from that

Some complexity lies in the details:
 - understanding the compact data present in tables
 - reuse of vertices from a previous cell when available
 - shrinking of the regular cells to make room for one or several transition cells. This is not systematic for all voxels of the cell (see `can_shrink`)
 - rotations: the algorithm is explained, and the tables are provided only for one transition face of the block (LowZ). For others, we need to apply a mapping to switch between XYZ and UVW coordinates

Terminology:
 - a `voxel` is a point on the densities grid. They are evenly spaced for a given resolution. Densities will be queried for voxels
 - a `grid point` is a  point ou our morphed grid. They will be used for output: a vertex is generated between 2 grid points
 For most regular cells, the 8 voxels and 8 grid points coincide.
For a regular cell touching one transition face of the block, 4 of the grid points will be shifted toward the inside of the block and not coincide with their origin voxels anymore
For a regular cell touching two transition faces, 6 such grid points are shifted away from their voxel
For a regular cell touching three transition faces, 7 grid points are shifted

Known room for improvements:
 - we never place a vertex exactly on a grid point. Since a vertex would only appear between greid points when one density is above the threshold and the other below, they should be different. If the densities are too close, we will place a verted in the middle of the grid points. As a consequence, we don't make use of TRANSITION_CORNER_DATA (illustrated in 4.19 in Lengyel's paper), which handles the reusing of previous cell vertex in the case the vertex is positionned exactly on a voxel
 - `transition_grid_point_on_low_res_face` calls `regular_grid_point` which will recalculate a grid point that was already calculated for the associated regular cell, and could be reused instead
 - actually grid points on the low res face always go in pairs (no case produces a vertex between the high res face and the low res face), and the vertex itself generated between them could be reused
 - a lot of things are probably copied, that should not
 */

use super::density_caching::PreCachingVoxelSource;

use super::super::mesh_builder::*;
use super::super::traits::*;
use super::super::transition_sides::*;
use super::super::voxel_coordinates::*;
use super::super::voxel_source::*;
use super::aux_tables::*;
use super::rotation::*;
use super::tables_wrapper::*;

/*
A point on the grid, for output purposes.
This is not necessarily at the same place as a Voxel sample, because grid points can be shifted ("shrink")
*/

pub struct Extractor<'b, C, V, S, M>
where
    C: Coordinate,
    V: VoxelData,
    S: VoxelSource<V>,
    M: MeshBuilder<V, C>,
{
    density_source: PreCachingVoxelSource<V, S>,
    block: &'b Block<C>,
    threshold: V::Density,
    transition_sides: TransitionSides,
    //vertices: usize,
    //vertices_positions: Vec<C>,
    //vertices_normals: Vec<C>,
    //tri_indices: Vec<usize>,
    mesh_builder: M,
    shared_storage: SharedVertexIndices,
    current_rotation: &'static Rotation,
}

impl<'b, C, V, S, M> Extractor<'b, C, V, S, M>
where
    C: Coordinate,
    V: VoxelData,
    S: VoxelSource<V>,
    M: MeshBuilder<V, C>,
{
    pub fn new(
        density_source: S,
        block: &'b Block<C>,
        threshold: V::Density,
        transition_sides: TransitionSides,
        mesh_builder: M,
    ) -> Self {
        Extractor::<'b, C, V, S, M> {
            density_source: PreCachingVoxelSource::new(density_source, block.subdivisions),
            block,
            threshold,
            transition_sides,
            //vertices: 0,
            mesh_builder,
            shared_storage: SharedVertexIndices::new(block.subdivisions),
            current_rotation: Rotation::default(),
        }
    }

    pub fn extract(mut self) -> M {
        self.extract_regular_cells();
        self.extract_transition_cells();
        self.mesh_builder
    }

    fn extract_regular_cells(&mut self) {
        for cell_x in 0..self.block.subdivisions {
            for cell_y in 0..self.block.subdivisions {
                for cell_z in 0..self.block.subdivisions {
                    let cell_index = RegularCellIndex {
                        x: cell_x,
                        y: cell_y,
                        z: cell_z,
                    };
                    self.extract_regular_cell(cell_index);
                }
            }
        }
    }

    fn extract_regular_cell(&mut self, cell_index: RegularCellIndex) {
        let case_number = self.regular_cell_case(&cell_index);
        let cell_class: u8 = transvoxel_data::regular_cell_data::REGULAR_CELL_CLASS[case_number];
        if cell_class != 0 {
            // To optimize, we could also check if the cell is on a border of the block, here
            // we only need voxels out of the block when such a cell generates vertices, because
            // these are for vertex normals
            self.density_source.load_regular_extended_voxels();
        }
        let triangulation_info =
            transvoxel_data::regular_cell_data::REGULAR_CELL_DATA[cell_class as usize];
        let vertices_data = transvoxel_data::regular_cell_data::REGULAR_VERTEX_DATA[case_number];
        let mut cell_vertices_indices: [VertexIndex; 12] = Default::default();
        for (i, vd) in vertices_data.iter().enumerate() {
            if i >= triangulation_info.get_vertex_count() as usize {
                break;
            }
            cell_vertices_indices[i] = self.regular_vertex(&cell_index, RegularVertexData(*vd));
        }
        for t in 0..triangulation_info.get_triangle_count() {
            let v1_index_in_cell = triangulation_info.vertex_index[3 * t as usize];
            let v2_index_in_cell = triangulation_info.vertex_index[3 * t as usize + 1];
            let v3_index_in_cell = triangulation_info.vertex_index[3 * t as usize + 2];
            let global_index_1 = cell_vertices_indices[v1_index_in_cell as usize];
            let global_index_2 = cell_vertices_indices[v2_index_in_cell as usize];
            let global_index_3 = cell_vertices_indices[v3_index_in_cell as usize];
            self.mesh_builder
                .add_triangle(global_index_1, global_index_2, global_index_3);
        }
    }

    fn extract_transition_cells(&mut self) {
        self.density_source
            .load_transition_voxels(self.transition_sides);
        for side in self.transition_sides {
            self.current_rotation = Rotation::for_side(side);
            for cell_u in 0..self.block.subdivisions {
                for cell_v in 0..self.block.subdivisions {
                    let cell_index = TransitionCellIndex::from(side, cell_u, cell_v);
                    self.extract_transition_cell(&cell_index);
                }
            }
        }
    }

    fn extract_transition_cell(&mut self, cell_index: &TransitionCellIndex) {
        let case_number = self.transition_cell_case(cell_index);
        let raw_cell_class =
            transvoxel_data::transition_cell_data::TRANSITION_CELL_CLASS[case_number];
        let cell_class = raw_cell_class & 0x7F;
        let invert_triangulation = (raw_cell_class & 0x80) != 0;
        let our_invert_triangulation = !invert_triangulation; // We use LowZ as base case so everything is inverted ?
        let triangulation_info =
            transvoxel_data::transition_cell_data::TRANSITION_CELL_DATA[cell_class as usize];
        let vertices_data =
            transvoxel_data::transition_cell_data::TRANSITION_VERTEX_DATA[case_number];
        let mut cell_vertices_indices: [VertexIndex; 12] = Default::default();
        for (i, vd) in vertices_data.iter().enumerate() {
            if i >= triangulation_info.get_vertex_count() as usize {
                break;
            }
            cell_vertices_indices[i] =
                self.transition_vertex(cell_index, TransitionVertexData(*vd));
        }
        for t in 0..triangulation_info.get_triangle_count() {
            let v1_index_in_cell = triangulation_info.vertex_index[3 * t as usize];
            let v2_index_in_cell = triangulation_info.vertex_index[3 * t as usize + 1];
            let v3_index_in_cell = triangulation_info.vertex_index[3 * t as usize + 2];
            let global_index_1 = cell_vertices_indices[v1_index_in_cell as usize];
            let global_index_2 = cell_vertices_indices[v2_index_in_cell as usize];
            let global_index_3 = cell_vertices_indices[v3_index_in_cell as usize];
            if our_invert_triangulation {
                self.mesh_builder
                    .add_triangle(global_index_1, global_index_2, global_index_3);
            } else {
                self.mesh_builder
                    .add_triangle(global_index_3, global_index_2, global_index_1);
            }
        }
    }

    fn regular_cell_case(&mut self, cell_index: &RegularCellIndex) -> usize {
        let mut case: usize = 0;
        for (i, deltas) in REGULAR_CELL_VOXELS.iter().enumerate() {
            let voxel_index = cell_index + deltas;
            let inside = self
                .regular_voxel_data(&voxel_index)
                .density()
                .inside(&self.threshold);
            if inside {
                case += 1 << i;
            }
        }
        case
    }

    fn transition_cell_case(&mut self, cell_index: &TransitionCellIndex) -> usize {
        let mut case: usize = 0;
        for (voxel_delta, contribution) in TRANSITION_HIGH_RES_FACE_CASE_CONTRIBUTIONS.iter() {
            let voxel_index = cell_index + voxel_delta;
            let density = self.transition_grid_point_data(&voxel_index).density();
            let inside = density.inside(&self.threshold);
            if inside {
                case += contribution;
            }
        }
        case
    }

    fn regular_voxel_data(&mut self, voxel_index: &RegularVoxelIndex) -> V {
        self.density_source.get_data(voxel_index)
    }

    fn transition_grid_point_data(&mut self, voxel_index: &HighResolutionVoxelIndex) -> V {
        if voxel_index.on_regular_grid() {
            let regular_index =
                voxel_index.as_regular_index(self.current_rotation, self.block.subdivisions);
            self.density_source.get_data(&regular_index)
        } else {
            self.density_source.get_transition_data(voxel_index)
        }
    }

    // Either creates or reuses an existing vertex. Returns its index in the vertices buffer
    fn regular_vertex(
        &mut self,
        cell_index: &RegularCellIndex,
        vd: RegularVertexData,
    ) -> VertexIndex {
        let cell_x = cell_index.x;
        let cell_y = cell_index.y;
        let cell_z = cell_index.z;
        if vd.new_vertex() {
            let i = self.new_regular_vertex(cell_index, vd.voxel_a_index(), vd.voxel_b_index());
            self.shared_storage
                .put_regular(i, cell_x, cell_y, cell_z, vd.reuse_index());
            i
        } else {
            let previous_vertex_is_accessible = ((vd.reuse_dx() == 0) || (cell_x > 0))
                && ((vd.reuse_dy() == 0) || (cell_y > 0))
                && ((vd.reuse_dz() == 0) || (cell_z > 0));
            if previous_vertex_is_accessible {
                self.shared_storage.get_regular(
                    (cell_x as isize + vd.reuse_dx()) as usize,
                    (cell_y as isize + vd.reuse_dy()) as usize,
                    (cell_z as isize + vd.reuse_dz()) as usize,
                    vd.reuse_index(),
                )
            } else {
                // We should reuse an existing vertex but its cell is not accessible (not part of our block)
                self.new_regular_vertex(cell_index, vd.voxel_a_index(), vd.voxel_b_index())
            }
        }
    }

    fn transition_vertex(
        &mut self,
        cell_index: &TransitionCellIndex,
        vd: TransitionVertexData,
    ) -> VertexIndex {
        if vd.reuse() {
            let cell_u = cell_index.cell_u;
            let cell_v = cell_index.cell_v;
            let previous_vertex_is_accessible =
                ((vd.reuse_du() == 0) || (cell_u > 0)) && ((vd.reuse_dv() == 0) || (cell_v > 0));
            if previous_vertex_is_accessible {
                let reuse_cell_u = (cell_u as isize + vd.reuse_du()) as usize;
                let reuse_cell_v = (cell_v as isize + vd.reuse_dv()) as usize;
                let previous_index = TransitionCellIndex {
                    side: cell_index.side,
                    cell_u: reuse_cell_u,
                    cell_v: reuse_cell_v,
                };
                self.shared_storage
                    .get_transition(&previous_index, vd.reuse_index())
            } else {
                self.new_transition_vertex(
                    cell_index,
                    vd.grid_point_a_index(),
                    vd.grid_point_b_index(),
                )
            }
        } else {
            let i = self.new_transition_vertex(
                cell_index,
                vd.grid_point_a_index(),
                vd.grid_point_b_index(),
            );
            if vd.new_reusable() {
                self.shared_storage
                    .put_transition(i, cell_index, vd.reuse_index());
            }
            i
        }
    }

    fn new_transition_vertex(
        &mut self,
        cell_index: &TransitionCellIndex,
        grid_point_a_index: TransitionCellGridPointIndex,
        grid_point_b_index: TransitionCellGridPointIndex,
    ) -> VertexIndex {
        let a = self.transition_grid_point(cell_index, grid_point_a_index);
        let b = self.transition_grid_point(cell_index, grid_point_b_index);
        self.add_vertex_between(a, b)
    }

    // Creates a new vertex. Returns its index in the vertices buffer
    fn new_regular_vertex(
        &mut self,
        cell_index: &RegularCellIndex,
        voxel_a_index: RegularCellVoxelIndex,
        voxel_b_index: RegularCellVoxelIndex,
    ) -> VertexIndex {
        let a = self.regular_grid_point_within_cell(cell_index, voxel_a_index);
        let b = self.regular_grid_point_within_cell(cell_index, voxel_b_index);
        self.add_vertex_between(a, b)
    }

    fn regular_grid_point_within_cell(
        &mut self,
        cell_index: &RegularCellIndex,
        voxel_index_within_cell: RegularCellVoxelIndex,
    ) -> GridPoint<V, C> {
        let voxel_deltas = get_regular_voxel_delta(voxel_index_within_cell);
        let voxel_index = cell_index + &voxel_deltas;
        self.regular_grid_point(voxel_index)
    }

    fn regular_grid_point(&mut self, voxel_index: RegularVoxelIndex) -> GridPoint<V, C> {
        let position = self.regular_grid_point_position(&voxel_index);
        let gradient = self.regular_voxel_gradient(&voxel_index);
        let voxel_data = self.regular_voxel_data(&voxel_index);
        GridPoint {
            position,
            gradient,
            voxel_data,
        }
    }

    fn regular_grid_point_position(&self, voxel_index: &RegularVoxelIndex) -> Position<C> {
        let mut x = self.block.dims.base[0]
            + self.block.dims.size * C::from_ratio(voxel_index.x, self.block.subdivisions);
        let mut y = self.block.dims.base[1]
            + self.block.dims.size * C::from_ratio(voxel_index.y, self.block.subdivisions);
        let mut z = self.block.dims.base[2]
            + self.block.dims.size * C::from_ratio(voxel_index.z, self.block.subdivisions);
        self.shrink_if_needed(&mut x, &mut y, &mut z, voxel_index);
        Position { x, y, z }
    }

    fn regular_voxel_gradient(
        &mut self,
        voxel_index: &RegularVoxelIndex,
    ) -> (V::Density, V::Density, V::Density) {
        let xgradient = self
            .regular_voxel_data(&(voxel_index + RegularVoxelDelta { x: 1, y: 0, z: 0 }))
            .density()
            .diff(
                self.regular_voxel_data(&(voxel_index + RegularVoxelDelta { x: -1, y: 0, z: 0 }))
                    .density(),
            );
        let ygradient = self
            .regular_voxel_data(&(voxel_index + RegularVoxelDelta { x: 0, y: 1, z: 0 }))
            .density()
            .diff(
                self.regular_voxel_data(&(voxel_index + RegularVoxelDelta { x: 0, y: -1, z: 0 }))
                    .density(),
            );
        let zgradient = self
            .regular_voxel_data(&(voxel_index + RegularVoxelDelta { x: 0, y: 0, z: 1 }))
            .density()
            .diff(
                self.regular_voxel_data(&(voxel_index + RegularVoxelDelta { x: 0, y: 0, z: -1 }))
                    .density(),
            );
        (xgradient, ygradient, zgradient)
    }

    fn transition_grid_point(
        &mut self,
        cell_index: &TransitionCellIndex,
        grid_point_index: TransitionCellGridPointIndex,
    ) -> GridPoint<V, C> {
        match TRANSITION_CELL_GRID_POINTS[grid_point_index.0] {
            TransitionCellGridPoint::HighResFace(delta) => {
                self.transition_grid_point_on_high_res_face(cell_index, delta)
            }
            TransitionCellGridPoint::RegularFace(face_u, face_v) => {
                self.transition_grid_point_on_low_res_face(cell_index, face_u, face_v)
            }
        }
    }

    fn transition_grid_point_on_low_res_face(
        &mut self,
        cell_index: &TransitionCellIndex,
        face_u: usize,
        face_v: usize,
    ) -> GridPoint<V, C> {
        let rot = self.current_rotation;
        let voxel_index =
            rot.to_regular_voxel_index(self.block.subdivisions, cell_index, face_u, face_v);
        self.regular_grid_point(voxel_index)
    }

    fn transition_grid_point_on_high_res_face(
        &mut self,
        cell_index: &TransitionCellIndex,
        delta: HighResolutionVoxelDelta,
    ) -> GridPoint<V, C> {
        let voxel_index = cell_index + &delta;
        let position = self.high_res_face_grid_point_position(cell_index, delta);
        let gradient = self.high_res_face_grid_point_gradient(&voxel_index);
        let voxel_data = self.high_res_face_grid_point_data(&voxel_index);
        GridPoint {
            position,
            gradient,
            voxel_data,
        }
    }

    fn high_res_face_grid_point_position(
        &self,
        cell_index: &TransitionCellIndex,
        delta: HighResolutionVoxelDelta,
    ) -> Position<C> {
        let rot = self.current_rotation;
        let voxel_index = cell_index + &delta;
        let position_in_block = rot.to_position_in_block(self.block.subdivisions, &voxel_index);
        &(&position_in_block * self.block.dims.size) + &self.block.dims.base
    }

    fn high_res_face_grid_point_gradient(
        &mut self,
        base_voxel_index: &HighResolutionVoxelIndex,
    ) -> (V::Density, V::Density, V::Density) {
        // This might not be correct, and we might want to only use high-res steps for the gradients,
        // even for voxels at the corners of the face (to better match normals with the neighbouring block)
        if base_voxel_index.on_regular_grid() {
            let regular_index =
                base_voxel_index.as_regular_index(self.current_rotation, self.block.subdivisions);
            self.regular_voxel_gradient(&regular_index)
        } else {
            self.high_res_face_grid_point_gradient_non_regular(base_voxel_index)
        }
    }

    fn high_res_face_grid_point_gradient_non_regular(
        &mut self,
        base_voxel_index: &HighResolutionVoxelIndex,
    ) -> (V::Density, V::Density, V::Density) {
        let rot = self.current_rotation;
        let x_gradient = self
            .transition_grid_point_data(&(base_voxel_index + &rot.plus_x_as_uvw))
            .density()
            .diff(
                self.transition_grid_point_data(&(base_voxel_index - &rot.plus_x_as_uvw))
                    .density(),
            );
        let y_gradient = self
            .transition_grid_point_data(&(base_voxel_index + &rot.plus_y_as_uvw))
            .density()
            .diff(
                self.transition_grid_point_data(&(base_voxel_index - &rot.plus_y_as_uvw))
                    .density(),
            );
        let z_gradient = self
            .transition_grid_point_data(&(base_voxel_index + &rot.plus_z_as_uvw))
            .density()
            .diff(
                self.transition_grid_point_data(&(base_voxel_index - &rot.plus_z_as_uvw))
                    .density(),
            );
        (x_gradient, y_gradient, z_gradient)
    }

    fn high_res_face_grid_point_data(&mut self, voxel_index: &HighResolutionVoxelIndex) -> V {
        self.transition_grid_point_data(voxel_index)
    }

    fn add_vertex_between(
        &mut self,
        point_a: GridPoint<V, C>,
        point_b: GridPoint<V, C>,
    ) -> VertexIndex {
        let interp_toward_b = V::Density::interp(
            point_a.voxel_data.density(),
            point_b.voxel_data.density(),
            self.threshold,
        );
        self.mesh_builder
            .add_vertex_between(point_a, point_b, interp_toward_b)
    }

    fn shrink_if_needed(
        &self,
        grid_point_x: &mut C,
        grid_point_y: &mut C,
        grid_point_z: &mut C,
        voxel_index: &RegularVoxelIndex,
    ) {
        let cell_size = self.block.dims.size * C::from_ratio(1, self.block.subdivisions);
        shrink_if_needed::<C>(
            grid_point_x,
            grid_point_y,
            grid_point_z,
            voxel_index.x,
            voxel_index.y,
            voxel_index.z,
            cell_size,
            self.block.subdivisions,
            &self.transition_sides,
        )
    }
}

// 0 to 3
/**
 Reuse index : ![Image](reuse_index.png)

Some code
```
let x = 1;
```

*/
struct SharedVertexIndices {
    regular: Vec<VertexIndex>,
    transition: Vec<VertexIndex>,
    block_size: usize,
}

impl SharedVertexIndices {
    pub fn new(block_size: usize) -> Self {
        SharedVertexIndices {
            regular: vec![VertexIndex(0); 4 * block_size * block_size * block_size], // 4 reusable vertex positions for each cell
            transition: vec![VertexIndex(0); 10 * 6 * block_size * block_size], // 10 reusable vertex positions potentially on each of the cell on each of the block sides
            block_size,
        }
    }
    pub fn get_regular(
        &self,
        cell_x: usize,
        cell_y: usize,
        cell_z: usize,
        reuse_index: RegularReuseIndex,
    ) -> VertexIndex {
        let storage_index = cell_x
            + self.block_size * cell_y
            + self.block_size * self.block_size * cell_z
            + self.block_size * self.block_size * self.block_size * reuse_index.0;
        self.regular[storage_index]
    }
    pub fn put_regular(
        &mut self,
        index: VertexIndex,
        cell_x: usize,
        cell_y: usize,
        cell_z: usize,
        reuse_index: RegularReuseIndex,
    ) {
        let storage_index = cell_x
            + self.block_size * cell_y
            + self.block_size * self.block_size * cell_z
            + self.block_size * self.block_size * self.block_size * reuse_index.0;
        self.regular[storage_index] = index;
    }
    pub fn get_transition(
        &self,
        cell: &TransitionCellIndex,
        reuse_index: TransitionReuseIndex,
    ) -> VertexIndex {
        let storage_index = cell.side as usize
            + 6 * cell.cell_u
            + 6 * self.block_size * cell.cell_v
            + 6 * self.block_size * self.block_size * reuse_index.0;
        self.transition[storage_index]
    }
    pub fn put_transition(
        &mut self,
        index: VertexIndex,
        cell: &TransitionCellIndex,
        reuse_index: TransitionReuseIndex,
    ) {
        let storage_index = cell.side as usize
            + 6 * cell.cell_u
            + 6 * self.block_size * cell.cell_v
            + 6 * self.block_size * self.block_size * reuse_index.0;
        self.transition[storage_index] = index;
    }
}

/// This function is only made public for our examples, to display the voxel grid. Regular users should not need it
#[allow(clippy::too_many_arguments)]
pub fn shrink_if_needed<C: Coordinate>(
    x: &mut C,
    y: &mut C,
    z: &mut C,
    xi: isize,
    yi: isize,
    zi: isize,
    cell_size: C,
    subdivisions: usize,
    transition_sides: &TransitionSides,
) {
    let shrink: C = C::shrink_factor() * cell_size;
    if can_shrink(xi, yi, zi, subdivisions, transition_sides) {
        if (xi == 0) && (transition_sides.contains(TransitionSide::LowX)) {
            *x = *x + shrink;
        } else if (xi as usize == subdivisions)
            && (transition_sides.contains(TransitionSide::HighX))
        {
            *x = *x - shrink;
        }
        if (yi == 0) && (transition_sides.contains(TransitionSide::LowY)) {
            *y = *y + shrink;
        } else if (yi as usize == subdivisions)
            && (transition_sides.contains(TransitionSide::HighY))
        {
            *y = *y - shrink;
        }
        if (zi == 0) && (transition_sides.contains(TransitionSide::LowZ)) {
            *z = *z + shrink;
        } else if (zi as usize == subdivisions)
            && (transition_sides.contains(TransitionSide::HighZ))
        {
            *z = *z - shrink;
        }
    }
}

// Do not shrink grid point (in any direction) if it's close to a face where the other block is rendered at the same
// (or lower) level of details (ie: not a transition side)
fn can_shrink(
    xi: isize,
    yi: isize,
    zi: isize,
    subdivisions: usize,
    transition_sides: &TransitionSides,
) -> bool {
    let dont_shrink = ((xi == 0) && !transition_sides.contains(TransitionSide::LowX))
        || ((xi == subdivisions as isize) && !transition_sides.contains(TransitionSide::HighX))
        || ((yi == 0) && !transition_sides.contains(TransitionSide::LowY))
        || ((yi == subdivisions as isize) && !transition_sides.contains(TransitionSide::HighY))
        || ((zi == 0) && !transition_sides.contains(TransitionSide::LowZ))
        || ((zi == subdivisions as isize) && !transition_sides.contains(TransitionSide::HighZ));
    !dont_shrink
}
