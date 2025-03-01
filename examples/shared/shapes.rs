use bevy::{
    asset::RenderAssetUsages,
    math::{primitives::Cuboid, Vec3},
    prelude::Transform,
    render::{
        mesh::{Indices, Mesh as BevyMesh, VertexAttributeValues},
        render_resource::PrimitiveTopology,
    },
};

pub fn create_arrow() -> BevyMesh {
    let shaft = BevyMesh::from(Cuboid::new(1.0, 0.1, 0.1));
    let head = BevyMesh::from(Cuboid::from_length(0.2));
    let arrow = merge(shaft, head);
    arrow
}

fn merge(mesh1: BevyMesh, mesh2: BevyMesh) -> BevyMesh {
    let transform2 = Transform::from_xyz(0.6, 0.0, 0.0);
    let transform1 = Transform::IDENTITY;

    let mut positions = Vec::<[f32; 3]>::new();
    append_f3(
        &mut positions,
        mesh1.attribute(BevyMesh::ATTRIBUTE_POSITION).unwrap(),
        &transform1,
    );
    append_f3(
        &mut positions,
        mesh2.attribute(BevyMesh::ATTRIBUTE_POSITION).unwrap(),
        &transform2,
    );
    let mut normals = Vec::<[f32; 3]>::new();
    append_f3(
        &mut normals,
        mesh1.attribute(BevyMesh::ATTRIBUTE_NORMAL).unwrap(),
        &transform1,
    );
    append_f3(
        &mut normals,
        mesh2.attribute(BevyMesh::ATTRIBUTE_NORMAL).unwrap(),
        &transform2,
    );
    let mut uvs = Vec::<[f32; 2]>::new();
    append_f2(&mut uvs, mesh1.attribute(BevyMesh::ATTRIBUTE_UV_0).unwrap());
    append_f2(&mut uvs, mesh2.attribute(BevyMesh::ATTRIBUTE_UV_0).unwrap());
    let indices2_shift = mesh1.count_vertices();
    let indices1 = mesh1.indices().unwrap();
    let indices = match indices1 {
        Indices::U16(is) => {
            let mut res = is.clone();
            match mesh2.indices().unwrap() {
                Indices::U16(iz) => {
                    for i in iz {
                        res.push(i + indices2_shift as u16);
                    }
                }
                Indices::U32(_) => {
                    panic!()
                }
            }
            Indices::U16(res)
        }
        Indices::U32(is) => {
            let mut res = is.clone();
            match mesh2.indices().unwrap() {
                Indices::U16(_) => {
                    panic!()
                }
                Indices::U32(iz) => {
                    for i in iz {
                        res.push(i + indices2_shift as u32);
                    }
                }
            }
            Indices::U32(res)
        }
    };

    let mut mesh = BevyMesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );
    mesh.insert_attribute(BevyMesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(BevyMesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(BevyMesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(indices);
    mesh
}

fn append_f3(dest: &mut Vec<[f32; 3]>, src: &VertexAttributeValues, transform: &Transform) -> () {
    if let VertexAttributeValues::Float32x3(values) = src {
        for value in values.iter() {
            let mut new_val = Vec3::from((value[0], value[1], value[2]));
            new_val = transform.transform_point(new_val);
            dest.push([new_val.x, new_val.y, new_val.z]);
        }
    } else {
        panic!()
    }
}

fn append_f2(dest: &mut Vec<[f32; 2]>, src: &VertexAttributeValues) -> () {
    if let VertexAttributeValues::Float32x2(values) = src {
        for value in values.iter() {
            dest.push(*value);
        }
    } else {
        panic!()
    }
}
