/*!
This module contains conversion functions toward [Bevy](https://bevyengine.org/) meshes 
*/
use bevy::render::mesh::Mesh as BevyMesh;
use crate::structs::Mesh as OurGenericMesh;
use bevy::render::render_resource::PrimitiveTopology::{LineList, TriangleList};

type OurMesh = OurGenericMesh<f32>;

/**
Convert to a Bevy mesh, producing a triangle list mesh with positions and normals
from our mesh, but UV coordinates all set to 0
*/
pub fn to_bevy(mesh: OurMesh) -> BevyMesh {
    let mut bevy_mesh = BevyMesh::new(TriangleList);
    let converted_indices: Vec<u32> =  mesh.triangle_indices.iter().map(|i| *i as u32).collect();
    let indices = bevy::render::mesh::Indices::U32(converted_indices);
    let n_vertex = mesh.positions.len() / 3;
    let uvs = vec![[0.0, 0.0]; n_vertex];

    let converted_positions = group_by_3(mesh.positions);
    let converted_normals = group_by_3(mesh.normals);

    bevy_mesh.set_indices(Some(indices));
    bevy_mesh.insert_attribute(BevyMesh::ATTRIBUTE_POSITION, converted_positions);
    bevy_mesh.insert_attribute(BevyMesh::ATTRIBUTE_NORMAL, converted_normals);
    bevy_mesh.insert_attribute(BevyMesh::ATTRIBUTE_UV_0, uvs);
    return bevy_mesh;
}

/**
Convert to a Bevy mesh lines list, with positions and normals
from our mesh, but UV coordinates all set to 0.
Lines shared between 2 triangles are repeated, for implementation simplicity.
*/
pub fn to_bevy_wireframe(mesh: OurMesh) -> BevyMesh {
    let mut bevy_mesh = BevyMesh::new(LineList);
    let indice_pairs = (0..mesh.num_tris())
        .map(|i| vec![3 * i, 3 * i + 1, 3 * i + 1, 3 * i + 2, 3 * i + 2, 3 * i]);
    let pairs_sequence = indice_pairs.flatten();
    let converted_indices: Vec<u32> = pairs_sequence
        .map(|i| mesh.triangle_indices[i] as u32)
        .collect();
    let indices = bevy::render::mesh::Indices::U32(converted_indices);
    let n_vertex = mesh.positions.len() / 3;
    let uvs = vec![[0.0, 0.0]; n_vertex];
    let converted_positions = group_by_3(mesh.positions);
    let converted_normals = group_by_3(mesh.normals);
    bevy_mesh.set_indices(Some(indices));
    bevy_mesh.insert_attribute(BevyMesh::ATTRIBUTE_POSITION, converted_positions);
    bevy_mesh.insert_attribute(BevyMesh::ATTRIBUTE_NORMAL, converted_normals);
    bevy_mesh.insert_attribute(BevyMesh::ATTRIBUTE_UV_0, uvs);
    return bevy_mesh;
}


fn group_by_3(source: Vec<f32>) -> Vec<[f32; 3]> {
    let len = source.len();
    assert!(len % 3 == 0);
    bytemuck::cast_slice(&source).to_vec()
}
