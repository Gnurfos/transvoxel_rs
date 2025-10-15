use crate::extraction::extract;
use crate::implementation::voxel_coordinates::RegularVoxelIndex;
use crate::structs::block::Block;
use crate::structs::block_star_view::BlockStarView;
use crate::structs::position::SamplingPosition;
use crate::traits::voxel_block::VoxelBlock;
use crate::{structs::generic_mesh::*, structs::transition_sides::*};
use hamcrest2::core::*;
use std::cell::RefCell;
use std::fmt::Debug;
use std::fmt::Display;
use std::ops::Sub;


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

impl Matcher<Triangle<f32>> for TriMatcher {
    fn matches(&self, actual: Triangle<f32>) -> MatchResult {
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
            Err(format!("{:?} not the same tri as {:?}", &actual, &self))
        }
    }
}

fn rotate(t: Triangle<f32>) -> Triangle<f32> {
    Triangle {
        vertices: [t.vertices[1], t.vertices[2], t.vertices[0]],
    }
}

fn same_pos(t1: Triangle<f32>, t2: Triangle<f32>) -> bool {
    (t1.vertices[0].position == t2.vertices[0].position)
        && (t1.vertices[1].position == t2.vertices[1].position)
        && (t1.vertices[2].position == t2.vertices[2].position)
}

fn same_pos_and_normal(t1: Triangle<f32>, t2: Triangle<f32>) -> bool {
    (t1.vertices[0] == t2.vertices[0])
        && (t1.vertices[1] == t2.vertices[1])
        && (t1.vertices[2] == t2.vertices[2])
}

#[allow(clippy::too_many_arguments)]
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
        x1,
        y1,
        z1,
        x2,
        y2,
        z2,
        x3,
        y3,
        z3,
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

#[allow(clippy::too_many_arguments)]
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
        x1,
        y1,
        z1,
        x2,
        y2,
        z2,
        x3,
        y3,
        z3,
        match_normals: true,
        nx1,
        ny1,
        nz1,
        nx2,
        ny2,
        nz2,
        nx3,
        ny3,
        nz3,
    }
}

#[allow(clippy::too_many_arguments)]
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

#[allow(clippy::too_many_arguments)]
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
    res
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
    tris.into_iter().filter(fully_in_cube).collect()
}

pub struct TestField {
    dense_points: Vec<(f32, f32, f32)>,
    block: Block<f32>,
    calls: RefCell<usize>,
}

impl TestField {
    pub fn new(base: [f32; 3], size: f32, subdivisions: usize) -> Self {
        Self {
            block: Block::new(base, size, subdivisions),
            dense_points: Vec::new(),
            calls: RefCell::new(0),
        }
    }
    pub fn set(&mut self, x: f32, y: f32, z: f32) {
        self.dense_points.push((x, y, z));
    }
    pub fn extract2(&self, threshold: f32, transition_sides: TransitionSides) -> GenericMeshBuilder<f32> {
        let b = GenericMeshBuilder::new();
        let field = |x, y, z| self.sample(x, y, z);
        let blocks = BlockStarView::new_relaying_to_field(&field, self.block, &transition_sides);
        extract(&blocks, threshold, b)
    }
    pub fn extract(&self, threshold: f32) -> GenericMeshBuilder<f32> {
        self.extract2(threshold, TransitionSide::none())
    }
    pub fn sample(&self, x: f32, y: f32, z: f32) -> f32 {
        *self.calls.borrow_mut() += 1;
        if self.dense_points.iter().any(|(px, py, pz)| (x.sub(px).abs() < f32::EPSILON) && (y.sub(py).abs() < f32::EPSILON) && (z.sub(pz).abs() < f32::EPSILON)) {
            1.0
        } else {
            0.0
        }
    }

}

impl VoxelBlock<f32, f32> for TestField {
    fn block(&self) -> &Block<f32> {
        &self.block
    }
    fn get(&self, index: RegularVoxelIndex) -> f32 {
        let SamplingPosition {x, y, z} = self.block.original_voxel_position(index);
        self.sample(x, y, z)
    }
}
