use std::cell::RefCell;
use std::env;

use crate::transition_sides::*;
use crate::unit_tests::test_utils::*;
use crate::{density::*, extraction::extract_from_fnmut};
use crate::{
    extraction::extract,
    voxel_source::{VoxelSource, WorldMappingVoxelSource},
};
use crate::{
    structs::*,
    voxel_coordinates::{HighResolutionVoxelIndex, RegularVoxelIndex},
};
use bevy::math::f32;
use flagset::Flags;
use hamcrest::prelude::*;
use hamcrest::*;
use rand::prelude::*;

#[test]
fn it_works() {
    assert_eq!(2 + 2, 4);
}

#[test]
fn empty_extraction() {
    let mut f = DensityArray::<f32>::new(10);
    let b = Block::from([0.0, 0.0, 0.0], 10.0, 10);
    let m = extract(&mut f, &b, 0.5, no_side());
    assert_that!(m.num_tris(), equal_to(0));
}

#[test]
fn one_cube_corner_gives_one_triangle() {
    let mut f = DensityArray::<f32>::new(1);
    f.set(0, 0, 0, 1f32);
    let b = Block::from([0.0, 0.0, 0.0], 1.0, 1);
    let m = extract(&mut f, &b, 0.5, no_side());
    assert_that!(
        m.tris(),
        tris!(tri_matcher(0.5, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5))
    );
}

#[test]
fn another_one_cube_corner() {
    let mut f = DensityArray::<f32>::new(3);
    f.set(3, 0, 0, 1f32);
    let b = Block::from([0.0, 0.0, 0.0], 3.0, 3);
    let m = extract(&mut f, &b, 0.5, no_side());
    assert_that!(
        m.tris(),
        tris!(tri_matcher(2.5, 0.0, 0.0, 3.0, 0.0, 0.5, 3.0, 0.5, 0.0))
    );
}

#[test]
fn two_corners_give_two_triangles_in_one_cube() {
    let mut f = DensityArray::<f32>::new(1);
    f.set(0, 0, 0, 1f32);
    f.set(1, 0, 0, 1f32);
    let b = Block::from([0.0, 0.0, 0.0], 1.0, 1);
    let m = extract(&mut f, &b, 0.5, no_side());
    assert_that!(
        m.tris(),
        tris!(
            tri_matcher(1.0, 0.0, 0.5, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5),
            tri_matcher(0.0, 0.5, 0.0, 1.0, 0.0, 0.5, 1.0, 0.5, 0.0)
        )
    );
}

#[test]
fn basic_normals() {
    // Extract a flat square (constant z), the normals should point to +Z
    let mut f = DensityArray::<f32>::new(1);
    for x in -1..3 {
        for y in -1..3 {
            f.set(x, y, 0, 1f32);
        }
    }
    let b = Block::from([0.0, 0.0, 0.0], 1.0, 1);
    let m = extract(&mut f, &b, 0.5, no_side());
    #[rustfmt::skip]
    assert_that!(
        m.tris(),
        tris!(
            tri_matcher_with_normals(
                0.0, 0.0, 0.5, 0.0, 0.0, 1.0,
                1.0, 0.0, 0.5, 0.0, 0.0, 1.0,
                1.0, 1.0, 0.5, 0.0, 0.0, 1.0),
            tri_matcher(
                0.0, 0.0, 0.5,
                1.0, 1.0, 0.5,
                0.0, 1.0, 0.5)
        )
    );
}

#[test]
fn ambiguous_case() {
    // First cell
    let mut f = DensityArray::<f32>::new(1);
    f.set(0, 0, 0, 1f32);
    f.set(0, 0, 1, 1f32);
    f.set(0, 1, 0, 1f32);
    f.set(0, 1, 1, 1f32);
    f.set(1, 1, 0, 1f32);
    f.set(1, 0, 1, 1f32);
    let b = Block::from([0.0, 0.0, 0.0], 1.0, 1);
    let m = extract(&mut f, &b, 0.5, no_side());
    assert_that!(m.tris().len(), equal_to(4));
    // Second cell (+x from the first one)
    let mut f = DensityArray::<f32>::new(1);
    f.set(0, 1, 0, 1f32);
    f.set(0, 0, 1, 1f32);
    let b = Block::from([1.0, 1.0, 1.0], 1.0, 1);
    let m = extract(&mut f, &b, 0.5, no_side());
    #[rustfmt::skip]
    assert_that!(
        m.tris(),
        tris!(
            tri_matcher(
                1.0, 1.5, 1.0,
                1.5, 2.0, 1.0,
                1.0, 2.0, 1.5),
            tri_matcher(
                1.0, 1.0, 1.5,
                1.0, 1.5, 2.0,
                1.5, 1.0, 2.0)
        )
    );
}

#[test]
fn vertices_are_reused_within_a_cell() {
    let mut f = DensityArray::<f32>::new(1);
    f.set(0, 0, 0, 1f32);
    f.set(1, 0, 0, 1f32);
    let b = Block::from([0.0, 0.0, 0.0], 1.0, 1);
    let m = extract(&mut f, &b, 0.5, no_side());
    assert_that!(m.tris().len(), equal_to(2));
    // 2 vertices should be reused to form a quad between the 2 triangles
    // => Total only 4 vertices instead of 6
    let positions_for_4_vertices = 4 * 3; // 4 * [x y z] in the positions buffer
    assert_that!(m.positions.len(), equal_to(positions_for_4_vertices));
}

#[test]
fn vertices_are_reused_between_cells() {
    let mut f = DensityArray::<f32>::new(2);
    f.set(0, 2, 2, 1f32);
    f.set(1, 2, 2, 1f32);
    let b = Block::from([0.0, 0.0, 0.0], 2.0, 2);
    let m = extract(&mut f, &b, 0.5, no_side());
    assert_that!(m.tris().len(), equal_to(3));
    // 2 vertices should be reused to form a quad between the 2 triangles of the first cell
    // 2 vertices should be reused too between cells
    // => Total only 5 vertices instead of 9 = 3x3tris
    let positions_for_5_vertices = 5 * 3; // 5 * [x y z] in the positions buffer
    assert_that!(m.positions.len(), equal_to(positions_for_5_vertices));
}

#[test]
fn trivial_transition_cell() {
    // Field left empty
    let mut f = DensityArray::<f32>::new(10);
    let b = Block::from([0.0, 0.0, 0.0], 10.0, 10);
    let transition_sides = TransitionSide::LowZ.into();
    let m = extract_from_grid(&mut f, &b, 0.5, transition_sides);
    assert_that!(m.tris().len(), equal_to(0));
}

#[test]
fn simplest_transition_cell() {
    let mut f = DensityArray::<f32>::new(10);
    // We need cells in the middle of a transition block face, to ensure actual cell shrinking, so the "simplest" is not so simple
    // This produces 4 "quarters" of a pyramid (viewed from z-top)
    //  q2 q1
    //  q3 q4
    f.set(5, 5, 0, 1f32);
    let b = Block::from([0.0, 0.0, 0.0], 100.0, 10);
    let transition_sides = TransitionSide::LowZ.into();
    let m = extract_from_grid(&mut f, &b, 0.5, transition_sides);
    // assert_that!(
    //     m.tris().len(),
    //     equal_to(12)
    // );
    // shrink = 0.15, cell size=10
    // 1.5 = transition width
    // 5.75 = transition width + half the remaining
    let v_top = (50.0, 50.0, 5.75);

    let q1v2 = (55f32, 50f32, 1.5f32); // Between the transition and the regular sub-cells
    let q1v3 = (50f32, 55f32, 1.5f32); // Between the transition and the regular sub-cells
    let q1v4 = (50f32, 52.5f32, 0f32); // On the high-res face
    let q1v5 = (52.5f32, 50f32, 0f32); // On the high-res face

    let q2v2 = (50f32, 55f32, 1.5f32);
    let q2v3 = (45f32, 50f32, 1.5f32);
    let q2v4 = (47.5f32, 50f32, 0f32);
    let q2v5 = (50f32, 52.5f32, 0f32);

    let q3v2 = (45f32, 50f32, 1.5f32);
    let q3v3 = (50f32, 45f32, 1.5f32);
    let q3v4 = (50f32, 47.5f32, 0f32);
    let q3v5 = (47.5f32, 50f32, 0f32);

    let q4v2 = (50f32, 45f32, 1.5f32);
    let q4v3 = (55f32, 50f32, 1.5f32);
    let q4v4 = (52.5f32, 50f32, 0f32);
    let q4v5 = (50f32, 47.5f32, 0f32);

    // Only Q1. This kind of tests the restrict function
    assert_that!(
        restrict(m.tris(), 50f32, 50f32, 0f32, 10f32),
        tris!(
            tri_matcher_vecs(v_top, q1v2, q1v3),
            tri_matcher_vecs(q1v4, q1v3, q1v2),
            tri_matcher_vecs(q1v2, q1v5, q1v4)
        )
    );

    assert_that!(
        m.tris(),
        tris!(
            // For each quarter (Q1):
            // Regular sub-cell has 1 triangle
            tri_matcher_vecs(v_top, q1v2, q1v3),
            // Transition sub-cell triangles
            tri_matcher_vecs(q1v4, q1v3, q1v2),
            tri_matcher_vecs(q1v2, q1v5, q1v4),
            // Q2
            tri_matcher_vecs(v_top, q2v2, q2v3),
            tri_matcher_vecs(q2v4, q2v3, q2v2),
            tri_matcher_vecs(q2v2, q2v5, q2v4),
            // Q3
            tri_matcher_vecs(v_top, q3v2, q3v3),
            tri_matcher_vecs(q3v4, q3v3, q3v2),
            tri_matcher_vecs(q3v2, q3v5, q3v4),
            // Q4
            tri_matcher_vecs(v_top, q4v2, q4v3),
            tri_matcher_vecs(q4v4, q4v3, q4v2),
            tri_matcher_vecs(q4v2, q4v5, q4v4)
        )
    );
}

#[test]
fn simple_transition_cell() {
    let mut f = DensityArray::<f32>::new(3);
    f.set(1, 1, 0, 1f32);
    // These go together, this is bad, they describe the same voxel requested by 2 different cells
    f.set_inter(TransitionSide::LowZ, 1, 1, 0, 1, 0, 1f32);
    f.set_inter(TransitionSide::LowZ, 0, 1, 2, 1, 0, 1f32);

    let b = Block::from([0.0, 0.0, 0.0], 30.0, 3);
    let transition_sides = TransitionSide::LowZ.into();
    let m = extract_from_grid(&mut f, &b, 0.5, transition_sides);
    let v_top = (10.0, 10.0, 5.75);
    let q1v2 = (15f32, 10f32, 1.5f32); // Between the transition and the regular sub-cells
    let q1v3 = (10f32, 15f32, 1.5f32); // Between the transition and the regular sub-cells
    let q1v4 = (12.5f32, 10f32, 0f32); // On the high-res face
    let q1v5 = (12.5f32, 15f32, 0f32); // On the high-res face
    let q1v6 = (10f32, 17.5f32, 0f32); // On the high-res face
    assert_that!(
        restrict(m.tris(), 10f32, 10f32, 0f32, 10f32),
        tris!(
            tri_matcher_vecs(v_top, q1v2, q1v3),
            tri_matcher_vecs(q1v2, q1v4, q1v5),
            tri_matcher_vecs(q1v5, q1v3, q1v2),
            tri_matcher_vecs(q1v5, q1v6, q1v3)
        )
    );
}

#[test]
fn simplest_transition_cell_non_negative_z() {
    let mut f = DensityArray::<f32>::new(3);
    f.set(0, 1, 1, 1f32);
    let b = Block::from([0.0, 0.0, 0.0], 30.0, 3);
    let transition_sides = TransitionSide::LowX.into();
    let m = extract_from_grid(&mut f, &b, 0.5, transition_sides);
    let v_top = (5.75, 10.0, 10.0);

    let q1v2 = (1.5f32, 15f32, 10f32); // Between the transition and the regular sub-cells
    let q1v3 = (1.5f32, 10f32, 15f32); // Between the transition and the regular sub-cells
    let q1v4 = (0f32, 10f32, 12.5f32); // On the high-res face
    let q1v5 = (0f32, 12.5f32, 10f32); // On the high-res face

    assert_that!(
        restrict(m.tris(), 0f32, 10f32, 10f32, 10f32),
        tris!(
            tri_matcher_vecs(v_top, q1v2, q1v3),
            tri_matcher_vecs(q1v4, q1v3, q1v2),
            tri_matcher_vecs(q1v2, q1v5, q1v4)
        )
    );
}

#[test]
fn simple_sphere() {
    // Centered on 10,10,10
    // Radius = 5, that is with threshold 0, density is 0 at 5 and 15, positive between them, negative outside
    struct Sphere;
    impl ScalarField<f32, f32> for Sphere {
        fn get_density(&self, x: f32, y: f32, z: f32) -> f32 {
            let distance_from_center =
                ((x - 10f32) * (x - 10f32) + (y - 10f32) * (y - 10f32) + (z - 10f32) * (z - 10f32))
                    .sqrt();
            let d = 1f32 - distance_from_center / 5f32;
            d
        }
    }
    let block = Block::from([0.0, 0.0, 0.0], 20.0, 2);
    let mut source = WorldMappingVoxelSource {
        field: &mut Sphere {},
        block: &block,
    };
    let threshold = 0f32;
    let m = extract(&mut source, &block, threshold, no_side());

    let v_plus_x = (15.0, 10.0, 10.0);
    let v_minus_x = (5.0, 10.0, 10.0);
    let v_plus_y = (10.0, 15.0, 10.0);
    let v_minus_y = (10.0, 5.0, 10.0);
    let v_plus_z = (10.0, 10.0, 15.0);
    let v_minus_z = (10.0, 10.0, 5.0);

    assert_that!(
        m.tris(),
        tris!(
            tri_matcher_vecs(v_plus_z, v_plus_x, v_plus_y),
            tri_matcher_vecs(v_plus_z, v_plus_y, v_minus_x),
            tri_matcher_vecs(v_plus_z, v_minus_x, v_minus_y),
            tri_matcher_vecs(v_plus_z, v_minus_y, v_plus_x),
            tri_matcher_vecs(v_minus_z, v_plus_x, v_minus_y),
            tri_matcher_vecs(v_minus_z, v_plus_y, v_plus_x),
            tri_matcher_vecs(v_minus_z, v_minus_x, v_plus_y),
            tri_matcher_vecs(v_minus_z, v_minus_y, v_minus_x)
        )
    );
}

struct CountingField<'b, S> {
    pub calls: RefCell<usize>,
    underlying: WorldMappingVoxelSource<'b, S, f32>,
}
impl<'b, C> CountingField<'b, ScalarFieldForFn<C>> {
    pub fn new(closure: C, block: &'b Block<f32>) -> Self {
        let underlying = WorldMappingVoxelSource::<'b, ScalarFieldForFn<C>, f32> {
            field: ScalarFieldForFn(closure),
            block: block,
        };
        Self {
            calls: RefCell::new(0),
            underlying: underlying,
        }
    }
    pub fn count(&self) -> usize {
        *self.calls.borrow()
    }
}
#[allow(unused_variables)]
impl<'b, S> VoxelSource<f32> for CountingField<'b, S>
where
    S: ScalarField<f32, f32>,
{
    fn get_density(&self, voxel_index: &RegularVoxelIndex) -> f32 {
        *self.calls.borrow_mut() += 1;
        self.underlying.get_density(voxel_index)
    }

    fn get_transition_density(&self, index: &HighResolutionVoxelIndex) -> f32 {
        *self.calls.borrow_mut() += 1;
        self.underlying.get_transition_density(index)
    }
}

#[test]
fn count_density_calls_minimal() {
    // Block with just 1 subdivision and empty
    let block = Block::from([0.0, 0.0, 0.0], 10.0, 1);
    // Regular
    let mut source = CountingField::new(|_, _, _| 0.0, &block);
    extract(&mut source, &block, 0.5, no_side());
    // Just query each voxel once for finding the case
    assert_that!(source.count(), equal_to(8));
    // With one transition
    let mut source = CountingField::new(|_, _, _| 0.0, &block);
    extract(&mut source, &block, 0.5, TransitionSide::LowX.into());
    // Just query each voxel once for finding the case, but needs the high-res- face voxels too
    assert_that!(source.count(), equal_to(13));
    // With two transition sides
    let mut source = CountingField::new(|_, _, _| 0.0, &block);
    extract(
        &mut source,
        &block,
        0.5,
        (TransitionSide::LowX | TransitionSide::LowZ).into(),
    );
    // Just query each voxel once for finding the case, but needs the 2 high-res face voxels too
    assert_that!(source.count(), equal_to(18));
}

#[test]
fn count_density_calls_random() {
    // Block with 3 subdivision and random
    let block = Block::from([0.0, 0.0, 0.0], 10.0, 3);
    // Regular
    let mut source = CountingField::new(&|_, _, _| rand::random(), &block);
    extract(&mut source, &block, 0.5, no_side());
    // Min: 4x4x4 for determining the case
    // Max: 4x4x4 for the case + 6x4x4 (extending outside in each of the 6 directions, for each of the  4x4 voxels on the side)
    assert_that!(source.count(), greater_than_or_equal_to(4 * 4 * 4));
    assert_that!(source.count(), less_than_or_equal_to(4 * 4 * 4 + 6 * 4 * 4));
    // With one transition
    let mut source = CountingField::new(&|_, _, _| rand::random(), &block);
    extract(&mut source, &block, 0.5, TransitionSide::LowX.into());
    // We're not very good here currently, difficult to assert
    // ...
}

#[test]
fn random_data() {
    // Just to get some small confidence that it doesn't crash
    let seed = match env::var("TEST_SEED") {
        Ok(s) => s.parse::<u64>().unwrap(),
        _ => random(),
    };
    println!("Using seed {}", seed);
    let mut rng = StdRng::seed_from_u64(seed);
    let subdivisions = rng.gen_range(2..30);
    let block = Block::from([0.0, 0.0, 0.0], 10.0, subdivisions);
    let sides = random_sides(&mut rng);
    let source = |_, _, _| rng.gen_range(-1.0..1.0);
    let m = extract_from_fnmut(source, &block, 0.5, sides);
    println!(
        "Extracted mesh with {} tris for sides {:?}",
        m.num_tris(),
        sides
    );
}

fn random_sides(rng: &mut StdRng) -> TransitionSides {
    let mut sides = no_side();
    for s in TransitionSide::LIST {
        if rng.gen_bool(0.5) {
            let ss: TransitionSides = (*s).into();
            sides = sides | ss;
        }
    }
    sides
}

#[test]
fn test_regular_cache_extended_loaded() {
    // A 3x3x3 block with low X as transition side (x == 0.0)
    // The only non-zero density is at (0.0, 1.5, 1.0), which means:
    //  -> the regular cell at 0, 1, 1 will produce nothing
    //  -> the corresponding transition cell (lowX, u=1, v=1) will produce something (case #2, 2 triangles)
    //  -> one of the produced vertices will need to evaluate the density gradient at (0.0, 1.0, 1.0), and
    //  for this access the density at (-1, 1, 1), which is on a the regular voxels grid. In earlier versions
    // of the aglorithm, this failed because the extended regular grid data was only loaded/accessible if a
    // regular cell needed it
    let subdivisions = 3;
    let size = 3.0;
    let block = Block::from([0.0, 0.0, 0.0], size, subdivisions);
    let sides = TransitionSide::LowX.into();
    let source = |x: f32, y: f32, z: f32| {
        if (x - 0.0).abs() > f32::EPSILON {
            0f32
        } else if (y - 1.5).abs() > f32::EPSILON {
            0f32
        } else if (z - 1.0).abs() > f32::EPSILON {
            0f32
        } else {
            1f32
        }
    };
    let m = extract_from_fnmut(source, &block, 0.5, sides);
    println!(
        "Extracted mesh with {} tris for sides {:?}",
        m.num_tris(),
        sides
    );
}
