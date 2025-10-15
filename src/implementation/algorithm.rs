use crate::structs::block_star_view::BlockStarView;
use crate::structs::grid_point::GridPoint;
use crate::structs::vertex_index::VertexIndex;
use crate::traits::coordinate::Coordinate;
use crate::traits::voxel_block::VoxelBlock;
use crate::traits::voxel_data::VoxelData;

use super::super::traits::mesh_builder::*;
use crate::implementation::voxel_coordinates::*;
use crate::structs::position::OutputPosition;
use super::aux_tables::*;
use super::rotation::*;
use super::tables_wrapper::*;
use crate::traits::voxel_data::Density;

pub struct Extractor<'b, C, V, CentralBlock, DenserNeighbourBlock, M>
where
    C: Coordinate,
    V: VoxelData,
    CentralBlock: VoxelBlock<C, V>,
    DenserNeighbourBlock: VoxelBlock<C, V>,
    M: MeshBuilder<V, C>,
{
    blocks: &'b BlockStarView<C, V, CentralBlock, DenserNeighbourBlock>,
    threshold: V::Density,
    mesh_builder: M,
    shared_storage: SharedVertexIndices,
    current_rotation: &'static Rotation,
}

impl<'b, C, V, CentralBlock, DenserNeighbourBlock, M> Extractor<'b, C, V, CentralBlock, DenserNeighbourBlock, M>
where
    C: Coordinate,
    V: VoxelData,
    CentralBlock: VoxelBlock<C, V>,
    DenserNeighbourBlock: VoxelBlock<C, V>,
    M: MeshBuilder<V, C>,
{
    pub fn new(
        blocks: &'b BlockStarView<C, V, CentralBlock, DenserNeighbourBlock>,
        threshold: V::Density,
        mesh_builder: M,
    ) -> Self {
        let subdivisions = blocks.central().block().subdivisions;
        Extractor::<'b, C, V, CentralBlock, DenserNeighbourBlock, M> {
            blocks,
            threshold,
            mesh_builder,
            shared_storage: SharedVertexIndices::new(subdivisions),
            current_rotation: Rotation::default(),
        }
    }

    pub fn extract(mut self) -> M {
        self.extract_regular_cells();
        self.extract_transition_cells();
        self.mesh_builder
    }

    fn extract_regular_cells(&mut self) {
        let subdivisions = self.blocks.central().block().subdivisions;
        for cell_x in 0..subdivisions {
            for cell_y in 0..subdivisions {
                for cell_z in 0..subdivisions {
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
        for side in self.blocks.transition_sides().into_iter() {
            self.current_rotation = Rotation::for_side(side);
            let subdivisions = self.blocks.central().block().subdivisions;
            for cell_u in 0..subdivisions {
                for cell_v in 0..subdivisions {
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
                .regular_voxel_data(voxel_index)
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
            let density = self.transition_grid_point_data(voxel_index).density();
            let inside = density.inside(&self.threshold);
            if inside {
                case += contribution;
            }
        }
        case
    }

    fn regular_voxel_data(&mut self, voxel_index: RegularVoxelIndex) -> V {
        self.blocks.central().get(voxel_index)
    }

    fn transition_grid_point_data(&mut self, voxel_index: HighResolutionVoxelIndex) -> V {
        let index_in_block = voxel_index.as_higher_res_neighbour_block_index(self.blocks.central().block().subdivisions);
        let block = self.blocks.neighbour_at(voxel_index.cell.side);
        block.get(index_in_block)
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
        let voxel_data = self.regular_voxel_data(voxel_index);
        GridPoint {
            position,
            gradient,
            voxel_data,
        }
    }

    fn regular_grid_point_position(&self, voxel_index: &RegularVoxelIndex) -> OutputPosition<C> {
        let central_block = self.blocks.central().block();
        central_block.morphed_voxel_position(*voxel_index, self.blocks.transition_sides())
    }

    fn regular_voxel_gradient(
        &mut self,
        voxel_index: &RegularVoxelIndex,
    ) -> (V::Density, V::Density, V::Density) {
        let xgradient = self
            .regular_voxel_data(voxel_index + RegularVoxelDelta { x: 1, y: 0, z: 0 })
            .density()
            .diff(
                self.regular_voxel_data(voxel_index + RegularVoxelDelta { x: -1, y: 0, z: 0 })
                    .density(),
            );
        let ygradient = self
            .regular_voxel_data(voxel_index + RegularVoxelDelta { x: 0, y: 1, z: 0 })
            .density()
            .diff(
                self.regular_voxel_data(voxel_index + RegularVoxelDelta { x: 0, y: -1, z: 0 })
                    .density(),
            );
        let zgradient = self
            .regular_voxel_data(voxel_index + RegularVoxelDelta { x: 0, y: 0, z: 1 })
            .density()
            .diff(
                self.regular_voxel_data(voxel_index + RegularVoxelDelta { x: 0, y: 0, z: -1 })
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
            rot.to_regular_voxel_index(self.blocks.central().block().subdivisions, cell_index, face_u, face_v);
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
        let voxel_data = self.high_res_face_grid_point_data(voxel_index);
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
    ) -> OutputPosition<C> {
        let rot = self.current_rotation;
        let voxel_index = cell_index + &delta;
        rot.to_position(*self.blocks.central().block(), &voxel_index)
    }

    fn high_res_face_grid_point_gradient(
        &mut self,
        base_voxel_index: &HighResolutionVoxelIndex,
    ) -> (V::Density, V::Density, V::Density) {
        // This might not be correct, and we might want to only use high-res steps for the gradients,
        // even for voxels at the corners of the face (to better match normals with the neighbouring block)
        if base_voxel_index.on_regular_grid() {
            let regular_index =
                base_voxel_index.as_regular_index(self.current_rotation, self.blocks.central().block().subdivisions);
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
            .transition_grid_point_data(base_voxel_index + &rot.plus_x_as_uvw)
            .density()
            .diff(
                self.transition_grid_point_data(base_voxel_index - &rot.plus_x_as_uvw)
                    .density(),
            );
        let y_gradient = self
            .transition_grid_point_data(base_voxel_index + &rot.plus_y_as_uvw)
            .density()
            .diff(
                self.transition_grid_point_data(base_voxel_index - &rot.plus_y_as_uvw)
                    .density(),
            );
        let z_gradient = self
            .transition_grid_point_data(base_voxel_index + &rot.plus_z_as_uvw)
            .density()
            .diff(
                self.transition_grid_point_data(base_voxel_index - &rot.plus_z_as_uvw)
                    .density(),
            );
        (x_gradient, y_gradient, z_gradient)
    }

    fn high_res_face_grid_point_data(&mut self, voxel_index: HighResolutionVoxelIndex) -> V {
        self.transition_grid_point_data(voxel_index)
    }

    fn add_vertex_between(
        &mut self,
        point_a: GridPoint<V, C>,
        point_b: GridPoint<V, C>,
    ) -> VertexIndex {
        let interpolate_toward_b = Density::interpolate(
            point_a.voxel_data.density(),
            point_b.voxel_data.density(),
            self.threshold,
        );
        self.mesh_builder
            .add_vertex_between(point_a, point_b, interpolate_toward_b)
    }

}

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

