use crate::models;
use bevy::render::mesh::Mesh as BevyMesh;
use transvoxel::shrink_if_needed;
use transvoxel::transition_sides::*;
use transvoxel::{
    bevy_mesh,
    density::ScalarField,
    extraction::extract,
    structs::{Block, Mesh as OurMesh},
    voxel_coordinates::{HighResolutionVoxelDelta, TransitionCellIndex},
    voxel_source::WorldMappingVoxelSource,
};

fn to_bevy(mesh: OurMesh, wireframe: bool) -> BevyMesh {
    if wireframe {
        bevy_mesh::to_bevy_wireframe(mesh)
    } else {
        bevy_mesh::to_bevy(mesh)
    }
}

pub fn mesh_for_model(
    model: &models::Model,
    wireframe: bool,
    block: &Block,
    transition_sides: &TransitionSides,
) -> BevyMesh {
    let mut models_map = models::models_map();
    let field = models_map.get_mut(model).unwrap().as_mut();
    field_model(field, wireframe, block, transition_sides)
}

pub fn inside_grid_points(
    model: &models::Model,
    block: &Block,
    transition_sides: &TransitionSides,
) -> Vec<(f32, f32, f32)> {
    let mut models_map = models::models_map();
    let field = models_map.get_mut(model).unwrap().as_mut();
    inside_grid_points_for_field(field, block, transition_sides)
}

fn field_model(
    field: &mut dyn ScalarField<f32>,
    wireframe: bool,
    block: &Block,
    transition_sides: &TransitionSides,
) -> BevyMesh {
    let mut source = WorldMappingVoxelSource {
        field: field,
        block: &block,
    };
    let mesh = extract(&mut source, &block, models::THRESHOLD, *transition_sides);
    to_bevy(mesh, wireframe)
}

fn inside_grid_points_for_field(
    field: &mut dyn ScalarField<f32>,
    block: &Block,
    transition_sides: &TransitionSides,
) -> Vec<(f32, f32, f32)> {
    let mut result = Vec::<(f32, f32, f32)>::new();
    let cell_size = block.dims.size / block.subdivisions as f32;
    // Regular points (some shrunk)
    for i in 0..=block.subdivisions {
        for j in 0..=block.subdivisions {
            for k in 0..=block.subdivisions {
                let unshrunk_pos = regular_position(block, i, j, k, &no_side());
                let final_pos = regular_position(block, i, j, k, transition_sides);
                let d = field.get_density(unshrunk_pos[0], unshrunk_pos[1], unshrunk_pos[2]);
                let inside = d >= models::THRESHOLD;
                if inside {
                    result.push((final_pos[0], final_pos[1], final_pos[2]));
                }
            }
        }
    }
    // Hig-res faces points
    for side in *transition_sides {
        for u in 0..=(block.subdivisions * 2) {
            for v in 0..=(block.subdivisions * 2) {
                let voxel_index = &TransitionCellIndex::from(side, 0, 0)
                    + &HighResolutionVoxelDelta::from(u as isize, v as isize, 0);
                let position_in_block = voxel_index.to_position_in_block(block);
                let pos = &(&position_in_block * cell_size) + &block.dims.base;
                let d = field.get_density(pos.x, pos.y, pos.z);
                let inside = d >= models::THRESHOLD;
                if inside {
                    result.push((pos.x, pos.y, pos.z));
                }
            }
        }
    }
    return result;
}

pub fn grid_lines(block: &Block, transition_sides: &TransitionSides) -> BevyMesh {
    let subs = block.subdivisions;
    let mut bevy_mesh = BevyMesh::new(bevy::render::pipeline::PrimitiveTopology::LineList);
    let mut positions = Vec::<[f32; 3]>::new();
    let mut indices = Vec::<u32>::new();
    for i in 0..=subs {
        for j in 0..=subs {
            // Z-line
            if subs == 1 {
                positions.push(regular_position(block, i, j, 0, transition_sides));
                positions.push(regular_position(block, i, j, 1, transition_sides));
            } else if subs == 2 {
                positions.push(regular_position(block, i, j, 0, transition_sides));
                positions.push(regular_position(block, i, j, 1, transition_sides));
                positions.push(regular_position(block, i, j, 1, transition_sides));
                positions.push(regular_position(block, i, j, 2, transition_sides));
            } else {
                positions.push(regular_position(block, i, j, 0, transition_sides));
                positions.push(regular_position(block, i, j, 1, transition_sides));
                positions.push(regular_position(block, i, j, 1, transition_sides));
                positions.push(regular_position(block, i, j, subs - 1, transition_sides));
                positions.push(regular_position(block, i, j, subs - 1, transition_sides));
                positions.push(regular_position(block, i, j, subs, transition_sides));
            }
            // Y-line
            if subs == 1 {
                positions.push(regular_position(block, i, 0, j, transition_sides));
                positions.push(regular_position(block, i, 1, j, transition_sides));
            } else if subs == 2 {
                positions.push(regular_position(block, i, 0, j, transition_sides));
                positions.push(regular_position(block, i, 1, j, transition_sides));
                positions.push(regular_position(block, i, 1, j, transition_sides));
                positions.push(regular_position(block, i, 2, j, transition_sides));
            } else {
                positions.push(regular_position(block, i, 0, j, transition_sides));
                positions.push(regular_position(block, i, 1, j, transition_sides));
                positions.push(regular_position(block, i, 1, j, transition_sides));
                positions.push(regular_position(block, i, subs - 1, j, transition_sides));
                positions.push(regular_position(block, i, subs - 1, j, transition_sides));
                positions.push(regular_position(block, i, subs, j, transition_sides));
            }
            // X-line
            if subs == 1 {
                positions.push(regular_position(block, 0, i, j, transition_sides));
                positions.push(regular_position(block, 1, i, j, transition_sides));
            } else if subs == 2 {
                positions.push(regular_position(block, 0, i, j, transition_sides));
                positions.push(regular_position(block, 1, i, j, transition_sides));
                positions.push(regular_position(block, 1, i, j, transition_sides));
                positions.push(regular_position(block, 2, i, j, transition_sides));
            } else {
                positions.push(regular_position(block, 0, i, j, transition_sides));
                positions.push(regular_position(block, 1, i, j, transition_sides));
                positions.push(regular_position(block, 1, i, j, transition_sides));
                positions.push(regular_position(block, subs - 1, i, j, transition_sides));
                positions.push(regular_position(block, subs - 1, i, j, transition_sides));
                positions.push(regular_position(block, subs, i, j, transition_sides));
            }
            // High res face lines
            for side in *transition_sides {
                for u_or_v in 0..=(subs * 2) {
                    // U-line
                    positions.push(high_res_face_grid_point_position(
                        block, side, 0, 0, 0, u_or_v,
                    ));
                    positions.push(high_res_face_grid_point_position(
                        block,
                        side,
                        subs - 1,
                        0,
                        2,
                        u_or_v,
                    ));
                    // V-line
                    positions.push(high_res_face_grid_point_position(
                        block, side, 0, 0, u_or_v, 0,
                    ));
                    positions.push(high_res_face_grid_point_position(
                        block,
                        side,
                        0,
                        subs - 1,
                        u_or_v,
                        2,
                    ));
                }
            }
            // Shafts from high-res face points to shrunk regular points
            for i in 0..=block.subdivisions {
                for j in 0..=block.subdivisions {
                    for k in 0..=block.subdivisions {
                        let unshrunk_pos = regular_position(block, i, j, k, &no_side());
                        let actual_pos = regular_position(block, i, j, k, transition_sides);
                        if unshrunk_pos != actual_pos {
                            positions.push(unshrunk_pos);
                            positions.push(actual_pos);
                        }
                    }
                }
            }
            // Indices
            for i in 0..positions.len() {
                indices.push(i as u32);
            }
        }
    }
    let normals = positions.clone(); // Not really important for lines ?
    let uvs = vec![[0.0, 0.0]; positions.len()];
    bevy_mesh.set_indices(Some(bevy::render::mesh::Indices::U32(indices)));
    bevy_mesh.set_attribute(BevyMesh::ATTRIBUTE_POSITION, positions);
    bevy_mesh.set_attribute(BevyMesh::ATTRIBUTE_NORMAL, normals);
    bevy_mesh.set_attribute(BevyMesh::ATTRIBUTE_UV_0, uvs);
    return bevy_mesh;
}

fn high_res_face_grid_point_position(
    block: &Block,
    side: TransitionSide,
    cell_u: usize,
    cell_v: usize,
    delta_u: usize,
    delta_v: usize,
) -> [f32; 3] {
    let cell_size = block.dims.size / block.subdivisions as f32;
    let voxel_index = &TransitionCellIndex::from(side, cell_u, cell_v)
        + &HighResolutionVoxelDelta::from(delta_u as isize, delta_v as isize, 0);
    let position_in_block = voxel_index.to_position_in_block(block);
    let pos = &(&position_in_block * cell_size) + &block.dims.base;
    [pos.x, pos.y, pos.z]
}

fn regular_position(
    block: &Block,
    cell_x: usize,
    cell_y: usize,
    cell_z: usize,
    transition_sides: &TransitionSides,
) -> [f32; 3] {
    let cell_size = block.dims.size / block.subdivisions as f32;
    let mut x = block.dims.base[0] + cell_x as f32 * cell_size;
    let mut y = block.dims.base[1] + cell_y as f32 * cell_size;
    let mut z = block.dims.base[2] + cell_z as f32 * cell_size;
    shrink_if_needed(
        &mut x,
        &mut y,
        &mut z,
        cell_x as isize,
        cell_y as isize,
        cell_z as isize,
        cell_size,
        block.subdivisions,
        transition_sides,
    );
    [x, y, z]
}
