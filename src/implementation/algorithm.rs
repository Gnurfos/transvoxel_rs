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

use super::super::density::*;
use super::super::structs::*;
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
struct GridPoint<D: Density> {
    position: Position<D::F>,
    gradient: (D::F, D::F, D::F),
    density: D,
}

pub struct Extractor<'b, D, S>
where
    D: Density,
    S: VoxelSource<D>,
{
    density_source: PreCachingVoxelSource<D, S>,
    block: &'b Block<D::F>,
    threshold: D,
    transition_sides: TransitionSides,
    vertices: usize,
    vertices_positions: Vec<D::F>,
    vertices_normals: Vec<D::F>,
    tri_indices: Vec<usize>,
    shared_storage: SharedVertexIndices,
    current_rotation: &'static Rotation,
}

impl<'b, D, S> Extractor<'b, D, S>
where
    D: Density,
    S: VoxelSource<D>,
{
    pub fn new(
        density_source: S,
        block: &'b Block<D::F>,
        threshold: D,
        transition_sides: TransitionSides,
    ) -> Self {
        Extractor::<'b, D, S> {
            density_source: PreCachingVoxelSource::new(density_source, block.subdivisions),
            block: block,
            threshold: threshold,
            transition_sides: transition_sides,
            vertices: 0,
            vertices_positions: Default::default(),
            vertices_normals: Default::default(),
            tri_indices: Default::default(),
            shared_storage: SharedVertexIndices::new(block.subdivisions),
            current_rotation: Rotation::default(),
        }
    }

    pub fn extract(mut self) -> Mesh<D::F> {
        self.extract_regular_cells();
        self.extract_transition_cells();
        return self.output_mesh();
    }

    fn output_mesh(self) -> Mesh<D::F> {
        return Mesh {
            positions: self.vertices_positions,
            normals: self.vertices_normals,
            triangle_indices: self.tri_indices,
        };
    }

    fn extract_regular_cells(&mut self) {
        for cell_x in 0..self.block.subdivisions {
            for cell_y in 0..self.block.subdivisions {
                for cell_z in 0..self.block.subdivisions {
                    let cell_index = RegularCellIndex { x: cell_x, y: cell_y, z: cell_z };
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
        let mut cell_vertices_indices: [usize; 12] = [0usize; 12];
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
            self.tri_indices.push(global_index_1);
            self.tri_indices.push(global_index_2);
            self.tri_indices.push(global_index_3);
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
        let case_number = self.transition_cell_case(&cell_index);
        let raw_cell_class =
            transvoxel_data::transition_cell_data::TRANSITION_CELL_CLASS[case_number];
        let cell_class = raw_cell_class & 0x7F;
        let invert_triangulation = (raw_cell_class & 0x80) != 0;
        let our_invert_triangulation = !invert_triangulation; // We use LowZ as base case so everything is inverted ?
        let triangulation_info =
            transvoxel_data::transition_cell_data::TRANSITION_CELL_DATA[cell_class as usize];
        let vertices_data =
            transvoxel_data::transition_cell_data::TRANSITION_VERTEX_DATA[case_number as usize];
        let mut cell_vertices_indices: [usize; 12] = [0usize; 12];
        for (i, vd) in vertices_data.iter().enumerate() {
            if i >= triangulation_info.get_vertex_count() as usize {
                break;
            }
            cell_vertices_indices[i] =
                self.transition_vertex(&cell_index, TransitionVertexData(*vd));
        }
        for t in 0..triangulation_info.get_triangle_count() {
            let v1_index_in_cell = triangulation_info.vertex_index[3 * t as usize];
            let v2_index_in_cell = triangulation_info.vertex_index[3 * t as usize + 1];
            let v3_index_in_cell = triangulation_info.vertex_index[3 * t as usize + 2];
            let global_index_1 = cell_vertices_indices[v1_index_in_cell as usize];
            let global_index_2 = cell_vertices_indices[v2_index_in_cell as usize];
            let global_index_3 = cell_vertices_indices[v3_index_in_cell as usize];
            if our_invert_triangulation {
                self.tri_indices.push(global_index_1);
                self.tri_indices.push(global_index_2);
                self.tri_indices.push(global_index_3);
            } else {
                self.tri_indices.push(global_index_3);
                self.tri_indices.push(global_index_2);
                self.tri_indices.push(global_index_1);
            }
        }
    }

    fn regular_cell_case(&mut self, cell_index: &RegularCellIndex) -> usize {
        let mut case: usize = 0;
        for (i, deltas) in REGULAR_CELL_VOXELS.iter().enumerate() {
            let voxel_index = cell_index + deltas;
            let inside = self.regular_voxel_density(&voxel_index).inside();
            if inside {
                case += 1 << i;
            }
        }
        return case;
    }

    fn transition_cell_case(&mut self, cell_index: &TransitionCellIndex) -> usize {
        let mut case: usize = 0;
        for (voxel_delta, contribution) in TRANSITION_HIGH_RES_FACE_CASE_CONTRIBUTIONS.iter() {
            let voxel_index = cell_index + voxel_delta;
            let density = self.transition_grid_point_density(&voxel_index);
            let inside = density.inside();
            if inside {
                case += contribution;
            }
        }
        return case;
    }

    fn regular_voxel_density(&mut self, voxel_index: &RegularVoxelIndex) -> D {
        self.density_source.get_density(voxel_index)
    }

    fn transition_grid_point_density(&mut self, voxel_index: &HighResolutionVoxelIndex) -> D {
        if voxel_index.on_regular_grid() {
            let regular_index =
                voxel_index.as_regular_index(self.current_rotation, self.block.subdivisions);
            self.density_source.get_density(&regular_index)
        } else {
            self.density_source.get_transition_density(voxel_index)
        }
    }

    // Either creates or reuses an existing vertex. Returns its index in the vertices buffer
    fn regular_vertex(&mut self, cell_index: &RegularCellIndex, vd: RegularVertexData) -> usize {
        let cell_x = cell_index.x;
        let cell_y = cell_index.y;
        let cell_z = cell_index.z;
        if vd.new_vertex() {
            let i = self.new_regular_vertex(&cell_index, vd.voxel_a_index(), vd.voxel_b_index());
            self.shared_storage
                .put_regular(i, cell_x, cell_y, cell_z, vd.reuse_index());
            return i;
        } else {
            let previous_vertex_is_accessible = ((vd.reuse_dx() == 0) || (cell_x > 0))
                && ((vd.reuse_dy() == 0) || (cell_y > 0))
                && ((vd.reuse_dz() == 0) || (cell_z > 0));
            if previous_vertex_is_accessible {
                return self.shared_storage.get_regular(
                    (cell_x as isize + vd.reuse_dx()) as usize,
                    (cell_y as isize + vd.reuse_dy()) as usize,
                    (cell_z as isize + vd.reuse_dz()) as usize,
                    vd.reuse_index(),
                );
            } else {
                // We should reuse an existing vertex but its cell is not accessible (not part of our block)
                let i =
                    self.new_regular_vertex(cell_index, vd.voxel_a_index(), vd.voxel_b_index());
                return i;
            }
        }
    }

    fn transition_vertex(
        &mut self,
        cell_index: &TransitionCellIndex,
        vd: TransitionVertexData,
    ) -> usize {
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
                return self
                    .shared_storage
                    .get_transition(&previous_index, vd.reuse_index());
            } else {
                let i = self.new_transition_vertex(
                    cell_index,
                    vd.grid_point_a_index(),
                    vd.grid_point_b_index(),
                );
                return i;
            }
        } else {
            let i =
                self.new_transition_vertex(cell_index, vd.grid_point_a_index(), vd.grid_point_b_index());
            if vd.new_reusable() {
                self.shared_storage
                    .put_transition(i, cell_index, vd.reuse_index());
            }
            return i;
        }
    }

    fn new_transition_vertex(
        &mut self,
        cell_index: &TransitionCellIndex,
        grid_point_a_index: TransitionCellGridPointIndex,
        grid_point_b_index: TransitionCellGridPointIndex,
    ) -> usize {
        let a = self.transition_grid_point(cell_index, grid_point_a_index);
        let b = self.transition_grid_point(cell_index, grid_point_b_index);
        let i = self.add_vertex_between(a, b);
        return i;
    }

    // Creates a new vertex. Returns its index in the vertices buffer
    fn new_regular_vertex(
        &mut self,
        cell_index: &RegularCellIndex,
        voxel_a_index: RegularCellVoxelIndex,
        voxel_b_index: RegularCellVoxelIndex,
    ) -> usize {
        let a = self.regular_grid_point_within_cell(cell_index, voxel_a_index);
        let b = self.regular_grid_point_within_cell(cell_index, voxel_b_index);
        return self.add_vertex_between(a, b);
    }

    fn regular_grid_point_within_cell(
        &mut self,
        cell_index: &RegularCellIndex,
        voxel_index_within_cell: RegularCellVoxelIndex,
    ) -> GridPoint<D> {
        let voxel_deltas = get_regular_voxel_delta(voxel_index_within_cell);
        let voxel_index = cell_index + &voxel_deltas;
        return self.regular_grid_point(voxel_index);
    }

    fn regular_grid_point(&mut self, voxel_index: RegularVoxelIndex) -> GridPoint<D> {
        let position = self.regular_grid_point_position(&voxel_index);
        let gradient = self.regular_voxel_gradient(&voxel_index);
        let density = self.regular_voxel_density(&voxel_index);
        return GridPoint {
            position: position,
            gradient: gradient,
            density: density,
        };
    }

    fn regular_grid_point_position(&self, voxel_index: &RegularVoxelIndex) -> Position<D::F> {
        let mut x = self.block.dims.base[0]
            + self.block.dims.size * D::F::from_ratio(voxel_index.x, self.block.subdivisions);
        let mut y = self.block.dims.base[1]
            + self.block.dims.size * D::F::from_ratio(voxel_index.y, self.block.subdivisions);
        let mut z = self.block.dims.base[2]
            + self.block.dims.size * D::F::from_ratio(voxel_index.z, self.block.subdivisions);
        self.shrink_if_needed(&mut x, &mut y, &mut z, &voxel_index);
        Position { x: x, y: y, z: z }
    }

    fn regular_voxel_gradient(&mut self, voxel_index: &RegularVoxelIndex) -> (D::F, D::F, D::F) {
        let xgradient =
            self.regular_voxel_density(&(voxel_index + RegularVoxelDelta { x: 1, y: 0, z: 0 }))
            .diff(self.regular_voxel_density(&(voxel_index + RegularVoxelDelta { x: -1, y: 0, z: 0 })));
        let ygradient =
            self.regular_voxel_density(&(voxel_index + RegularVoxelDelta { x: 0, y: 1, z: 0 }))
            .diff(self.regular_voxel_density(&(voxel_index + RegularVoxelDelta { x: 0, y: -1, z: 0 })));
        let zgradient =
            self.regular_voxel_density(&(voxel_index + RegularVoxelDelta { x: 0, y: 0, z: 1 }))
            .diff(self.regular_voxel_density(&(voxel_index + RegularVoxelDelta { x: 0, y: 0, z: -1 })));
        (xgradient, ygradient, zgradient)
    }

    fn transition_grid_point(
        &mut self,
        cell_index: &TransitionCellIndex,
        grid_point_index: TransitionCellGridPointIndex,
    ) -> GridPoint<D> {
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
    ) -> GridPoint<D> {
        let rot = self.current_rotation;
        let voxel_index =
            rot.to_regular_voxel_index(self.block.subdivisions, cell_index, face_u, face_v);
        return self.regular_grid_point(voxel_index);
    }

    fn transition_grid_point_on_high_res_face(
        &mut self,
        cell_index: &TransitionCellIndex,
        delta: HighResolutionVoxelDelta,
    ) -> GridPoint<D> {
        let voxel_index = cell_index + &delta;
        let position = self.high_res_face_grid_point_position(cell_index, delta);
        let gradient = self.high_res_face_grid_point_gradient(&voxel_index);
        let density = self.high_res_face_grid_point_density(&voxel_index);
        return GridPoint {
            position: position,
            gradient: gradient,
            density: density,
        };
    }

    fn high_res_face_grid_point_position(
        &self,
        cell_index: &TransitionCellIndex,
        delta: HighResolutionVoxelDelta,
    ) -> Position<D::F> {
        let rot = self.current_rotation;
        let voxel_index = cell_index + &delta;
        let position_in_block = rot.to_position_in_block(self.block.subdivisions, &voxel_index);
        let world_position = &(&position_in_block * self.block.dims.size) + &self.block.dims.base;
        world_position
    }

    fn high_res_face_grid_point_gradient(
        &mut self,
        base_voxel_index: &HighResolutionVoxelIndex,
    ) -> (D::F, D::F, D::F) {
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
    ) -> (D::F, D::F, D::F) {
        let rot = self.current_rotation;
        let x_gradient =
            self.transition_grid_point_density(&(base_voxel_index + &rot.plus_x_as_uvw))
            .diff(self.transition_grid_point_density(&(base_voxel_index - &rot.plus_x_as_uvw)));
        let y_gradient =
            self.transition_grid_point_density(&(base_voxel_index + &rot.plus_y_as_uvw))
            .diff(self.transition_grid_point_density(&(base_voxel_index - &rot.plus_y_as_uvw)));
        let z_gradient =
            self.transition_grid_point_density(&(base_voxel_index + &rot.plus_z_as_uvw))
            .diff(self.transition_grid_point_density(&(base_voxel_index - &rot.plus_z_as_uvw)));
        (x_gradient, y_gradient, z_gradient)
    }

    fn high_res_face_grid_point_density(&mut self, voxel_index: &HighResolutionVoxelIndex) -> D {
        self.transition_grid_point_density(&voxel_index)
    }

    fn add_vertex_between(&mut self, point_a: GridPoint<D>, point_b: GridPoint<D>) -> usize {
        let interp_toward_b = D::interp(&point_a.density, &point_b.density, &self.threshold);
        let position = point_a
            .position
            .interp_toward(&point_b.position, interp_toward_b);
        let gradient_x =
            point_a.gradient.0 + interp_toward_b * (point_b.gradient.0 - point_a.gradient.0);
        let gradient_y =
            point_a.gradient.1 + interp_toward_b * (point_b.gradient.1 - point_a.gradient.1);
        let gradient_z =
            point_a.gradient.2 + interp_toward_b * (point_b.gradient.2 - point_a.gradient.2);
        let normal = D::to_normal(&gradient_x, &gradient_y, &gradient_z);
        self.vertices_positions.push(position.x);
        self.vertices_positions.push(position.y);
        self.vertices_positions.push(position.z);
        self.vertices_normals.push(normal[0]);
        self.vertices_normals.push(normal[1]);
        self.vertices_normals.push(normal[2]);
        let index = self.vertices;
        self.vertices += 1;
        return index;
    }

    fn shrink_if_needed(
        &self,
        grid_point_x: &mut D::F,
        grid_point_y: &mut D::F,
        grid_point_z: &mut D::F,
        voxel_index: &RegularVoxelIndex,
    ) {
        let cell_size = self.block.dims.size * D::F::from_ratio(1, self.block.subdivisions);
        shrink_if_needed::<D>(
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
    regular: Vec<usize>,
    transition: Vec<usize>,
    block_size: usize,
}

impl SharedVertexIndices {
    pub fn new(block_size: usize) -> Self {
        SharedVertexIndices {
            regular: vec![0; 4 * block_size * block_size * block_size], // 4 reusable vertex positions for each cell
            transition: vec![0; 10 * 6 * block_size * block_size], // 10 reusable vertex positions potentially on each of the cell on each of the block sides
            block_size: block_size,
        }
    }
    pub fn get_regular(
        &self,
        cell_x: usize,
        cell_y: usize,
        cell_z: usize,
        reuse_index: RegularReuseIndex,
    ) -> usize {
        let storage_index = cell_x
            + self.block_size * cell_y
            + self.block_size * self.block_size * cell_z
            + self.block_size * self.block_size * self.block_size * reuse_index.0;
        self.regular[storage_index]
    }
    pub fn put_regular(
        &mut self,
        index: usize,
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
    ) -> usize {
        let storage_index = cell.side as usize
            + 6 * cell.cell_u
            + 6 * self.block_size * cell.cell_v
            + 6 * self.block_size * self.block_size * reuse_index.0;
        self.transition[storage_index]
    }
    pub fn put_transition(
        &mut self,
        index: usize,
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
pub fn shrink_if_needed<D: Density>(
    x: &mut D::F,
    y: &mut D::F,
    z: &mut D::F,
    xi: isize,
    yi: isize,
    zi: isize,
    cell_size: D::F,
    subdivisions: usize,
    transition_sides: &TransitionSides,
) {
    let shrink: D::F = D::shrink_factor() * cell_size;
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
    return !dont_shrink;
}
