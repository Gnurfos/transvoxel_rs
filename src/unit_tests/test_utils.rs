use crate::structs::*;
use crate::{density::*, transition_sides::*, voxel_source::VoxelSource};
use crate::{
    extraction::extract,
    voxel_coordinates::{HighResolutionVoxelIndex, RegularVoxelIndex},
};
use ndarray::{Array3, Array6};
use std::fmt::Debug;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub struct TriMatcher {
    x1: f32,
    y1: f32,
    z1: f32,
    x2: f32,
    y2: f32,
    z2: f32,
    x3: f32,
    y3: f32,
    z3: f32,
    match_normals: bool,
    nx1: f32,
    ny1: f32,
    nz1: f32,
    nx2: f32,
    ny2: f32,
    nz2: f32,
    nx3: f32,
    ny3: f32,
    nz3: f32,
}

impl Display for TriMatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Triangle like:")?;
        if self.match_normals {
            writeln!(
                f,
                "    + Pos [{:?}, {:?}, {:?}] Norm [{:?}, {:?}, {:?}]",
                self.x1, self.y1, self.z1, self.nx1, self.ny1, self.nz1
            )?;
            writeln!(
                f,
                "    + Pos [{:?}, {:?}, {:?}] Norm [{:?}, {:?}, {:?}]",
                self.x2, self.y2, self.z2, self.nx2, self.ny2, self.nz2
            )?;
            writeln!(
                f,
                "    + Pos [{:?}, {:?}, {:?}] Norm [{:?}, {:?}, {:?}]",
                self.x3, self.y3, self.z3, self.nx3, self.ny3, self.nz3
            )?;
            Ok(())
        } else {
            writeln!(f, "    + Pos [{:?}, {:?}, {:?}]", self.x1, self.y1, self.z1)?;
            writeln!(f, "    + Pos [{:?}, {:?}, {:?}]", self.x2, self.y2, self.z2)?;
            writeln!(f, "    + Pos [{:?}, {:?}, {:?}]", self.x3, self.y3, self.z3)?;
            Ok(())
        }
    }
}

impl hamcrest::core::Matcher<Triangle<f32>> for TriMatcher {
    fn matches(&self, actual: Triangle<f32>) -> hamcrest::core::MatchResult {
        let tris_match = if self.match_normals {
            same_pos_and_normal
        } else {
            same_pos
        };
        #[rustfmt::skip]
        let base_tri = if self.match_normals {
            make_tri_with_normals(
                self.x1, self.y1, self.z1, self.nx1, self.ny1, self.nz1,
                self.x2, self.y2, self.z2, self.nx2, self.ny2, self.nz2,
                self.x3, self.y3, self.z3, self.nx3, self.ny3, self.nz3,
            )
        } else {
            make_tri(
                self.x1, self.y1, self.z1, self.x2, self.y2, self.z2, self.x3, self.y3, self.z3,
            )
        };
        let rotated_1 = rotate(base_tri);
        let rotated_2 = rotate(rotated_1);
        let same = tris_match(actual, base_tri)
            || tris_match(actual, rotated_1)
            || tris_match(actual, rotated_2);
        if same {
            success()
        } else {
            return Err(format!("{:?} not the same tri as {:?}", &actual, &self));
        }
    }
}

fn rotate(t: Triangle<f32>) -> Triangle<f32> {
    Triangle {
        vertices: [t.vertices[1], t.vertices[2], t.vertices[0]],
    }
}

fn same_pos(t1: Triangle<f32>, t2: Triangle<f32>) -> bool {
    return (t1.vertices[0].position == t2.vertices[0].position)
        && (t1.vertices[1].position == t2.vertices[1].position)
        && (t1.vertices[2].position == t2.vertices[2].position);
}

fn same_pos_and_normal(t1: Triangle<f32>, t2: Triangle<f32>) -> bool {
    return (t1.vertices[0] == t2.vertices[0])
        && (t1.vertices[1] == t2.vertices[1])
        && (t1.vertices[2] == t2.vertices[2]);
}

pub fn tri_matcher(
    x1: f32,
    y1: f32,
    z1: f32,
    x2: f32,
    y2: f32,
    z2: f32,
    x3: f32,
    y3: f32,
    z3: f32,
) -> TriMatcher {
    TriMatcher {
        x1: x1,
        y1: y1,
        z1: z1,
        x2: x2,
        y2: y2,
        z2: z2,
        x3: x3,
        y3: y3,
        z3: z3,
        match_normals: false,
        nx1: 0.0,
        ny1: 0.0,
        nz1: 0.0,
        nx2: 0.0,
        ny2: 0.0,
        nz2: 0.0,
        nx3: 0.0,
        ny3: 0.0,
        nz3: 0.0,
    }
}

pub fn tri_matcher_vecs(
    v1: (f32, f32, f32),
    v2: (f32, f32, f32),
    v3: (f32, f32, f32),
) -> TriMatcher {
    tri_matcher(v1.0, v1.1, v1.2, v2.0, v2.1, v2.2, v3.0, v3.1, v3.2)
}

pub fn tri_matcher_with_normals(
    x1: f32,
    y1: f32,
    z1: f32,
    nx1: f32,
    ny1: f32,
    nz1: f32,
    x2: f32,
    y2: f32,
    z2: f32,
    nx2: f32,
    ny2: f32,
    nz2: f32,
    x3: f32,
    y3: f32,
    z3: f32,
    nx3: f32,
    ny3: f32,
    nz3: f32,
) -> TriMatcher {
    TriMatcher {
        x1: x1,
        y1: y1,
        z1: z1,
        x2: x2,
        y2: y2,
        z2: z2,
        x3: x3,
        y3: y3,
        z3: z3,
        match_normals: true,
        nx1: nx1,
        ny1: ny1,
        nz1: nz1,
        nx2: nx2,
        ny2: ny2,
        nz2: nz2,
        nx3: nx3,
        ny3: ny3,
        nz3: nz3,
    }
}

pub fn make_tri(
    x1: f32,
    y1: f32,
    z1: f32,
    x2: f32,
    y2: f32,
    z2: f32,
    x3: f32,
    y3: f32,
    z3: f32,
) -> Triangle<f32> {
    Triangle {
        vertices: [
            Vertex {
                position: [x1, y1, z1],
                normal: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [x2, y2, z2],
                normal: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [x3, y3, z3],
                normal: [0.0, 0.0, 0.0],
            },
        ],
    }
}

pub fn make_tri_with_normals(
    x1: f32,
    y1: f32,
    z1: f32,
    nx1: f32,
    ny1: f32,
    nz1: f32,
    x2: f32,
    y2: f32,
    z2: f32,
    nx2: f32,
    ny2: f32,
    nz2: f32,
    x3: f32,
    y3: f32,
    z3: f32,
    nx3: f32,
    ny3: f32,
    nz3: f32,
) -> Triangle<f32> {
    Triangle {
        vertices: [
            Vertex {
                position: [x1, y1, z1],
                normal: [nx1, ny1, nz1],
            },
            Vertex {
                position: [x2, y2, z2],
                normal: [nx2, ny2, nz2],
            },
            Vertex {
                position: [x3, y3, z3],
                normal: [nx3, ny3, nz3],
            },
        ],
    }
}

#[derive(Debug)]
pub struct TrianglesMatcher {
    pub items: Vec<TriMatcher>,
}

impl Display for TrianglesMatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Triangles: ")?;
        for item in self.items.iter() {
            writeln!(f, " - {}", item)?;
        }
        Ok(())
    }
}

use hamcrest::core::*;
impl Matcher<Vec<Triangle<f32>>> for TrianglesMatcher {
    fn matches(&self, actual: Vec<Triangle<f32>>) -> MatchResult {
        let mut rem = actual.clone();

        for item in self.items.iter() {
            match rem.iter().position(|a| item.matches(*a) == Ok(())) {
                Some(idx) => {
                    rem.remove(idx);
                }
                None => {
                    let formatted_actual = format_list(actual);
                    return Err(format!("was:\n{}", &formatted_actual));
                }
            }
        }

        if !rem.is_empty() {
            let formatted_remaining = format_list(rem);
            return Err(format!("also had {}\n", formatted_remaining));
        }

        success()
    }
}

fn format_list<T: Display>(list: Vec<T>) -> String {
    let mut res = String::from("");
    for item in list.iter() {
        res += " - ";
        res += &format!("{}", item);
    }
    return res;
}

macro_rules! tris {
    () => (
        $crate::unit_tests::test_utils::TrianglesMatcher {items: vec!()}
    );
    ($($x:expr),*) => (
        $crate::unit_tests::test_utils::TrianglesMatcher {items: vec!($($x),*)}
    );
}

pub fn restrict(
    tris: Vec<Triangle<f32>>,
    min_x: f32,
    min_y: f32,
    min_z: f32,
    size: f32,
) -> Vec<Triangle<f32>> {
    let in_cube = |v: &Vertex<f32>| -> bool {
        (v.position[0] >= min_x)
            && (v.position[0] <= min_x + size)
            && (v.position[1] >= min_y)
            && (v.position[1] <= min_y + size)
            && (v.position[2] >= min_z)
            && (v.position[2] <= min_z + size)
    };
    let fully_in_cube = |tri: &Triangle<f32>| {
        in_cube(&tri.vertices[0]) && in_cube(&tri.vertices[1]) && in_cube(&tri.vertices[2])
    };
    return tris.into_iter().filter(fully_in_cube).collect();
}

/**
BLOCK_SIZE means there are that many cells across every dimension
ex: BLOCK_SIZE=3 means a 3x3x3 cells block, thus storing 4x4x4 density points for the block itself
The points outside of this (-1 and BLOCK_SIZE + 1 can be accessed for gradients) are also allocated, resulting in a 5x5x5 array
*/

pub struct DensityArray<D>
where
    D: Density + Default,
{
    data: Array3<D>,
    inter_data: Array6<D>,
}

impl<D> DensityArray<D>
where
    D: Density + Default,
{
    pub fn set(&mut self, cell_x: isize, cell_y: isize, cell_z: isize, value: D) {
        self.data[[
            (cell_x + 1) as usize,
            (cell_y + 1) as usize,
            (cell_z + 1) as usize,
        ]] = value;
    }

    pub fn set_inter(
        &mut self,
        side: TransitionSide,
        cell_u: usize,
        cell_v: usize,
        du: isize,
        dv: isize,
        dw: isize,
        value: D,
    ) {
        let i = [
            side as usize,
            cell_u,
            cell_v,
            (du + 1) as usize,
            (dv + 1) as usize,
            (dw + 1) as usize,
        ];
        self.inter_data[i] = value;
    }

    fn get_inter(
        &self,
        side: TransitionSide,
        cell_u: usize,
        cell_v: usize,
        du: isize,
        dv: isize,
        dw: isize,
    ) -> D {
        let i = [
            side as usize,
            cell_u,
            cell_v,
            (du + 1) as usize,
            (dv + 1) as usize,
            (dw + 1) as usize,
        ];
        self.inter_data[i]
    }
}

impl<D: Default + Density + Copy> VoxelSource<D> for DensityArray<D> {
    fn get_density(&self, voxel_index: &RegularVoxelIndex) -> D {
        self.data[[
            (voxel_index.x + 1) as usize,
            (voxel_index.y + 1) as usize,
            (voxel_index.z + 1) as usize,
        ]]
    }

    #[allow(unused_variables, unused_mut)]
    fn get_transition_density(&self, index: &HighResolutionVoxelIndex) -> D {
        let side = index.cell.side;
        if (index.delta.u % 2 != 0) || (index.delta.v % 2 != 0) {
            return self.get_inter(
                side,
                index.cell.cell_u,
                index.cell.cell_v,
                index.delta.u,
                index.delta.v,
                index.delta.w,
            );
        }
        // Both u and v were even. We should only be called for non regular voxels, so w should be non-zero
        assert!(
            index.delta.w != 0,
            "get_transition_density was called for a regular voxel"
        );
        // This is for computing a gradient
        // Then return whatever because we don't care about normals in this implementation
        return D::default();
    }
}

impl<D: Default + Density> DensityArray<D> {
    pub fn new(block_size: usize) -> Self {
        DensityArray {
            data: Array3::<D>::default((block_size + 3, block_size + 3, block_size + 3)),
            inter_data: Array6::<D>::default((6, block_size, block_size, 5, 5, 5)),
        }
    }
}

pub fn extract_from_grid<D: Density + Default + Copy>(
    field: &mut DensityArray<D>,
    block: &Block<D::F>,
    threshold: D,
    transition_sides: TransitionSides,
) -> Mesh<D::F> {
    extract(field, block, threshold, transition_sides)
}
