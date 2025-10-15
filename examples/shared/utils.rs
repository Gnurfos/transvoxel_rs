use crate::bevy_mesh::BevyMeshBuilder;
use crate::models;
use bevy::asset::RenderAssetUsages;
use bevy::math::Quat;
use bevy::math::Vec3;
use bevy::mesh::Indices;
use bevy::prelude::Mesh as BevyMesh;
use bevy::transform::components::Transform;
use transvoxel::extraction::extract;
use transvoxel::private::_internal_high_res_voxel_index_to_position;
use transvoxel::prelude::BlockStarView;
use transvoxel::structs::block::Block;
use transvoxel::structs::position::OutputPosition;
use transvoxel::structs::transition_sides::*;
use transvoxel::structs::voxel_index::VoxelIndex;
use transvoxel::traits::data_field::DataField;

#[allow(unused)]
pub fn mesh_for_model(
    model: &models::Model,
    wireframe: bool,
    block: Block<f32>,
    transition_sides: &TransitionSides,
) -> BevyMesh {
    let mut models_map = models::models_map();
    let field = models_map.get_mut(model).unwrap().as_mut();
    field_model(field, wireframe, block, transition_sides)
}

pub fn inside_grid_points(
    model: &models::Model,
    block: &Block<f32>,
    transition_sides: &TransitionSides,
) -> Vec<(f32, f32, f32)> {
    let mut models_map = models::models_map();
    let field = models_map.get_mut(model).unwrap().as_mut();
    inside_grid_points_for_field(field, block, transition_sides)
}

#[allow(unused)]
fn field_model(
    field: &mut dyn DataField<f32, f32>,
    wireframe: bool,
    block: Block<f32>,
    transition_sides: &TransitionSides,
) -> BevyMesh {
    let mut blocks = BlockStarView::new_relaying_to_field(field, block, transition_sides);
    let builder = extract(
        &mut blocks,
        models::THRESHOLD,
        BevyMeshBuilder::default(),
    );
    if wireframe {
        builder.build_wireframe()
    } else {
        builder.build()
    }
}

fn inside_grid_points_for_field(
    field: &mut dyn DataField<f32, f32>,
    block: &Block<f32>,
    transition_sides: &TransitionSides,
) -> Vec<(f32, f32, f32)> {
    let mut result = Vec::<(f32, f32, f32)>::new();
    // Regular points (some shrunk)
    for i in 0..=block.subdivisions {
        for j in 0..=block.subdivisions {
            for k in 0..=block.subdivisions {
                let index = VoxelIndex { x: i as isize, y: j as isize, z: k as isize};
                let unshrunk_pos = block.original_voxel_position(index);
                let final_pos = block.morphed_voxel_position(index, transition_sides);
                let d = field.get_data(unshrunk_pos.x, unshrunk_pos.y, unshrunk_pos.z);
                let inside = d >= models::THRESHOLD;
                if inside {
                    result.push((final_pos.x, final_pos.y, final_pos.z));
                }
            }
        }
    }
    // Hig-res faces points
    for side in *transition_sides {
        for u in 0..=(block.subdivisions * 2) {
            for v in 0..=(block.subdivisions * 2) {
                let pos = _internal_high_res_voxel_index_to_position(
                    *block,
                    side,
                    0,
                    0,
                    u as isize,
                    v as isize,
                    0
                );
                let d = field.get_data(pos.x, pos.y, pos.z);
                let inside = d >= models::THRESHOLD;
                if inside {
                    result.push((pos.x, pos.y, pos.z));
                }
            }
        }
    }
    result
}

pub fn grid_line_mesh() -> BevyMesh {
    let mut bevy_mesh = BevyMesh::new(
        bevy::render::render_resource::PrimitiveTopology::LineList,
        RenderAssetUsages::default(),
    );
    let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
    let indices = vec![0, 1];
    bevy_mesh.insert_indices(Indices::U32(indices));
    bevy_mesh.insert_attribute(BevyMesh::ATTRIBUTE_POSITION, positions);
    bevy_mesh
}
pub fn grid_line_transforms(block: &Block<f32>, transition_sides: &TransitionSides) -> Vec<Transform> {
    let subs = block.subdivisions;
    let mut points = Vec::new();
    for i in 0..=subs {
        for j in 0..=subs {
            // Z-line
            if subs == 1 {
                points.push(regular_position(block, i, j, 0, transition_sides));
                points.push(regular_position(block, i, j, 1, transition_sides));
            } else if subs == 2 {
                points.push(regular_position(block, i, j, 0, transition_sides));
                points.push(regular_position(block, i, j, 1, transition_sides));
                points.push(regular_position(block, i, j, 1, transition_sides));
                points.push(regular_position(block, i, j, 2, transition_sides));
            } else {
                points.push(regular_position(block, i, j, 0, transition_sides));
                points.push(regular_position(block, i, j, 1, transition_sides));
                points.push(regular_position(block, i, j, 1, transition_sides));
                points.push(regular_position(block, i, j, subs - 1, transition_sides));
                points.push(regular_position(block, i, j, subs - 1, transition_sides));
                points.push(regular_position(block, i, j, subs, transition_sides));
            }
            // Y-line
            if subs == 1 {
                points.push(regular_position(block, i, 0, j, transition_sides));
                points.push(regular_position(block, i, 1, j, transition_sides));
            } else if subs == 2 {
                points.push(regular_position(block, i, 0, j, transition_sides));
                points.push(regular_position(block, i, 1, j, transition_sides));
                points.push(regular_position(block, i, 1, j, transition_sides));
                points.push(regular_position(block, i, 2, j, transition_sides));
            } else {
                points.push(regular_position(block, i, 0, j, transition_sides));
                points.push(regular_position(block, i, 1, j, transition_sides));
                points.push(regular_position(block, i, 1, j, transition_sides));
                points.push(regular_position(block, i, subs - 1, j, transition_sides));
                points.push(regular_position(block, i, subs - 1, j, transition_sides));
                points.push(regular_position(block, i, subs, j, transition_sides));
            }
            // X-line
            if subs == 1 {
                points.push(regular_position(block, 0, i, j, transition_sides));
                points.push(regular_position(block, 1, i, j, transition_sides));
            } else if subs == 2 {
                points.push(regular_position(block, 0, i, j, transition_sides));
                points.push(regular_position(block, 1, i, j, transition_sides));
                points.push(regular_position(block, 1, i, j, transition_sides));
                points.push(regular_position(block, 2, i, j, transition_sides));
            } else {
                points.push(regular_position(block, 0, i, j, transition_sides));
                points.push(regular_position(block, 1, i, j, transition_sides));
                points.push(regular_position(block, 1, i, j, transition_sides));
                points.push(regular_position(block, subs - 1, i, j, transition_sides));
                points.push(regular_position(block, subs - 1, i, j, transition_sides));
                points.push(regular_position(block, subs, i, j, transition_sides));
            }
            // High-res face lines
            for side in *transition_sides {
                for u_or_v in 0..=(subs * 2) {
                    // U-line
                    points.push(high_res_face_grid_point_position(
                        block, side, 0, 0, 0, u_or_v,
                    ));
                    points.push(high_res_face_grid_point_position(
                        block,
                        side,
                        subs - 1,
                        0,
                        2,
                        u_or_v,
                    ));
                    // V-line
                    points.push(high_res_face_grid_point_position(
                        block, side, 0, 0, u_or_v, 0,
                    ));
                    points.push(high_res_face_grid_point_position(
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
                        let unshrunk_pos = regular_position(block, i, j, k, &TransitionSide::none());
                        let actual_pos = regular_position(block, i, j, k, transition_sides);
                        if unshrunk_pos != actual_pos {
                            points.push(unshrunk_pos);
                            points.push(actual_pos);
                        }
                    }
                }
            }
        }
    }
    points
        .chunks(2)
        .map(|c| transform_from_x(&c[0], &c[1]))
        .collect()
}

fn transform_from_x(a: &OutputPosition<f32>, b: &OutputPosition<f32>) -> Transform {
    use bevy::math::NormedVectorSpace;
    let a = Vec3::from_array([a.x, a.y, a.z]);
    let b = Vec3::from_array([b.x, b.y, b.z]);
    Transform {
        translation: a,
        rotation: Quat::from_rotation_arc(Vec3::X, (b - a).normalize()),
        scale: Vec3::splat((b - a).norm()),
    }
}

#[test]
fn test_transform() {
    use bevy::math::NormedVectorSpace;
    let a = Vec3::new(1.0, 1.0, 1.0);
    let b = Vec3::new(11.0, 11.0, 11.0);
    let t = Transform {
        translation: a,
        rotation: Quat::from_rotation_arc(Vec3::X, (b - a).normalize()),
        scale: Vec3::splat((b - a).norm()),
    };
    let transformed_o = t.transform_point(Vec3::ZERO);
    let transformed_x = t.transform_point(Vec3::X);
    println!("{transformed_o}");
    println!("{transformed_x}");
    assert_eq!(transformed_o, a);
    assert_eq!(transformed_x, b);
}

// pub fn grid_lines(block: &Block<f32>, transition_sides: &TransitionSides) -> BevyMesh {
//     let subs = block.subdivisions;
//     let mut bevy_mesh = BevyMesh::new(
//         bevy::render::render_resource::PrimitiveTopology::LineList,
//         RenderAssetUsages::default(),
//     );
//     let mut positions = Vec::<[f32; 3]>::new();
//     let mut indices = Vec::<u32>::new();
//     for i in 0..=subs {
//         for j in 0..=subs {
//             // Z-line
//             if subs == 1 {
//                 positions.push(regular_position(block, i, j, 0, transition_sides));
//                 positions.push(regular_position(block, i, j, 1, transition_sides));
//             } else if subs == 2 {
//                 positions.push(regular_position(block, i, j, 0, transition_sides));
//                 positions.push(regular_position(block, i, j, 1, transition_sides));
//                 positions.push(regular_position(block, i, j, 1, transition_sides));
//                 positions.push(regular_position(block, i, j, 2, transition_sides));
//             } else {
//                 positions.push(regular_position(block, i, j, 0, transition_sides));
//                 positions.push(regular_position(block, i, j, 1, transition_sides));
//                 positions.push(regular_position(block, i, j, 1, transition_sides));
//                 positions.push(regular_position(block, i, j, subs - 1, transition_sides));
//                 positions.push(regular_position(block, i, j, subs - 1, transition_sides));
//                 positions.push(regular_position(block, i, j, subs, transition_sides));
//             }
//             // Y-line
//             if subs == 1 {
//                 positions.push(regular_position(block, i, 0, j, transition_sides));
//                 positions.push(regular_position(block, i, 1, j, transition_sides));
//             } else if subs == 2 {
//                 positions.push(regular_position(block, i, 0, j, transition_sides));
//                 positions.push(regular_position(block, i, 1, j, transition_sides));
//                 positions.push(regular_position(block, i, 1, j, transition_sides));
//                 positions.push(regular_position(block, i, 2, j, transition_sides));
//             } else {
//                 positions.push(regular_position(block, i, 0, j, transition_sides));
//                 positions.push(regular_position(block, i, 1, j, transition_sides));
//                 positions.push(regular_position(block, i, 1, j, transition_sides));
//                 positions.push(regular_position(block, i, subs - 1, j, transition_sides));
//                 positions.push(regular_position(block, i, subs - 1, j, transition_sides));
//                 positions.push(regular_position(block, i, subs, j, transition_sides));
//             }
//             // X-line
//             if subs == 1 {
//                 positions.push(regular_position(block, 0, i, j, transition_sides));
//                 positions.push(regular_position(block, 1, i, j, transition_sides));
//             } else if subs == 2 {
//                 positions.push(regular_position(block, 0, i, j, transition_sides));
//                 positions.push(regular_position(block, 1, i, j, transition_sides));
//                 positions.push(regular_position(block, 1, i, j, transition_sides));
//                 positions.push(regular_position(block, 2, i, j, transition_sides));
//             } else {
//                 positions.push(regular_position(block, 0, i, j, transition_sides));
//                 positions.push(regular_position(block, 1, i, j, transition_sides));
//                 positions.push(regular_position(block, 1, i, j, transition_sides));
//                 positions.push(regular_position(block, subs - 1, i, j, transition_sides));
//                 positions.push(regular_position(block, subs - 1, i, j, transition_sides));
//                 positions.push(regular_position(block, subs, i, j, transition_sides));
//             }
//             // High res face lines
//             for side in *transition_sides {
//                 for u_or_v in 0..=(subs * 2) {
//                     // U-line
//                     positions.push(high_res_face_grid_point_position(
//                         block, side, 0, 0, 0, u_or_v,
//                     ));
//                     positions.push(high_res_face_grid_point_position(
//                         block,
//                         side,
//                         subs - 1,
//                         0,
//                         2,
//                         u_or_v,
//                     ));
//                     // V-line
//                     positions.push(high_res_face_grid_point_position(
//                         block, side, 0, 0, u_or_v, 0,
//                     ));
//                     positions.push(high_res_face_grid_point_position(
//                         block,
//                         side,
//                         0,
//                         subs - 1,
//                         u_or_v,
//                         2,
//                     ));
//                 }
//             }
//             // Shafts from high-res face points to shrunk regular points
//             for i in 0..=block.subdivisions {
//                 for j in 0..=block.subdivisions {
//                     for k in 0..=block.subdivisions {
//                         let unshrunk_pos = regular_position(block, i, j, k, &no_side());
//                         let actual_pos = regular_position(block, i, j, k, transition_sides);
//                         if unshrunk_pos != actual_pos {
//                             positions.push(unshrunk_pos);
//                             positions.push(actual_pos);
//                         }
//                     }
//                 }
//             }
//             // Indices
//             for i in 0..positions.len() {
//                 indices.push(i as u32);
//             }
//         }
//     }
//     let normals = positions.clone(); // Not really important for lines ?
//     bevy_mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));
//     bevy_mesh.insert_attribute(BevyMesh::ATTRIBUTE_POSITION, positions);
//     bevy_mesh.insert_attribute(BevyMesh::ATTRIBUTE_NORMAL, normals);
//     bevy_mesh
// }

fn high_res_face_grid_point_position(
    block: &Block<f32>,
    side: TransitionSide,
    cell_u: usize,
    cell_v: usize,
    delta_u: usize,
    delta_v: usize,
) -> OutputPosition<f32> {
    let pos = _internal_high_res_voxel_index_to_position(
        *block,
        side,
        cell_u,
        cell_v,
        delta_u as isize,
        delta_v as isize,
        0
    );
    pos
}

fn regular_position(block: &Block<f32>, i: usize, j: usize, k: usize, transition_sides: &TransitionSides) -> OutputPosition<f32> {
    block.morphed_voxel_position(VoxelIndex { x: i as isize, y: j as isize, z: k as isize }, transition_sides)
}