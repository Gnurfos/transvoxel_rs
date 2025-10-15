use std::cell::RefCell;
use std::env;

use crate::extraction::extract;
use crate::structs::generic_mesh::*;
use crate::structs::transition_sides::*;
use crate::unit_tests::test_utils::*;
use bevy::math::f32;
use flagset::Flags;
use hamcrest2::prelude::*;
use rand::prelude::*;
use rand::rng;
use rand::Rng;
use crate::structs::block::Block;
use crate::structs::block_star_view::BlockStarView;
use crate::traits::data_field::DataField;


#[test]
fn empty_extraction() {
    let f = TestField::new([0.0, 0.0, 0.0], 10.0, 10);
    let m = f.extract(0.5).build();
    assert_that!(m.num_tris(), equal_to(0));
}

#[test]
fn one_cube_corner_gives_one_triangle() {
    let mut f = TestField::new([0.0, 0.0, 0.0], 1.0, 1);
    f.set(0.0, 0.0, 0.0);
    let m = f.extract(0.5).build();
    assert_that!(
        m.tris(),
        tris!(tri_matcher(0.5, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5))
    );
}



#[test]
fn another_one_cube_corner() {
    let mut f = TestField::new([0.0, 0.0, 0.0], 3.0, 3);
    f.set(3.0, 0.0, 0.0);
    let m = f.extract(0.5).build();
    assert_that!(
        m.tris(),
        tris!(tri_matcher(2.5, 0.0, 0.0, 3.0, 0.0, 0.5, 3.0, 0.5, 0.0))
    );
}

#[test]
fn two_corners_give_two_triangles_in_one_cube() {
    let mut f = TestField::new([0.0, 0.0, 0.0], 1.0, 1);
    f.set(0.0, 0.0, 0.0);
    f.set(1.0, 0.0, 0.0);
    let m = f.extract(0.5).build();
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
    let mut f = TestField::new([0.0, 0.0, 0.0], 1.0, 1);
    for x in -1..3 {
        for y in -1..3 {
            f.set(x as f32, y as f32, 0.0);
        }
    }
    let m = f.extract(0.5).build();
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
    let mut f = TestField::new([0.0, 0.0, 0.0], 1.0, 1);
    f.set(0.0, 0.0, 0.0);
    f.set(0.0, 0.0, 1.0);
    f.set(0.0, 1.0, 0.0);
    f.set(0.0, 1.0, 1.0);
    f.set(1.0, 1.0, 0.0);
    f.set(1.0, 0.0, 1.0);
    let m = f.extract(0.5).build();
    assert_that!(m.tris().len(), equal_to(4));
    // Second cell (+x+y+z from the first one)
    let mut f = TestField::new([1.0, 1.0, 1.0], 1.0, 1);
    f.set(1.0, 2.0, 1.0);
    f.set(1.0, 1.0, 2.0);
    let m = f.extract(0.5).build();
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
    let mut f = TestField::new([0.0, 0.0, 0.0], 1.0, 1);
    f.set(0.0, 0.0, 0.0);
    f.set(1.0, 0.0, 0.0);
    let m = f.extract(0.5).build();
    assert_that!(m.tris().len(), equal_to(2));
    // 2 vertices should be reused to form a quad between the 2 triangles
    // => Total only 4 vertices instead of 6
    let positions_for_4_vertices = 4 * 3; // 4 * [x y z] in the positions buffer
    assert_that!(m.positions.len(), equal_to(positions_for_4_vertices));
}

#[test]
fn vertices_are_reused_between_cells() {
    let mut f = TestField::new([0.0, 0.0, 0.0], 2.0, 2);
    f.set(0.0, 2.0, 2.0);
    f.set(1.0, 2.0, 2.0);
    let m = f.extract(0.5).build();
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
    let f = TestField::new([0.0, 0.0, 0.0], 10.0, 10);
    let m = f.extract2(0.5, TransitionSide::LowZ.into()).build();
    assert_that!(m.tris().len(), equal_to(0));
}

#[test]
fn simplest_transition_cell() {
    let mut f = TestField::new([0.0, 0.0, 0.0], 100.0, 10);
    // We need cells in the middle of a transition block face, to ensure actual cell shrinking, so the "simplest" is not so simple
    // This produces 4 "quarters" of a pyramid (viewed from z-top)
    //  q2 q1
    //  q3 q4
    f.set(50.0, 50.0, 0.0);
    let m = f.extract2(0.5, TransitionSide::LowZ.into()).build();
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
    let mut f = TestField::new([0.0, 0.0, 0.0], 30.0, 3);
    f.set(10.0, 10.0, 0.0);
    f.set(10.0, 15.0, 0.0);
    let m = f.extract2(0.5, TransitionSide::LowZ.into()).build();
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
    let mut f = TestField::new([0.0, 0.0, 0.0], 30.0, 3);
    f.set(0.0, 10.0, 10.0);
    let m = f.extract2(0.5, TransitionSide::LowX.into()).build();
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
    impl DataField<f32, f32> for Sphere {
        fn get_data(&self, x: f32, y: f32, z: f32) -> f32 {
            let distance_from_center =
                ((x - 10f32) * (x - 10f32) + (y - 10f32) * (y - 10f32) + (z - 10f32) * (z - 10f32))
                    .sqrt();
            1f32 - distance_from_center / 5f32
        }
    }
    let block = Block::new([0.0, 0.0, 0.0], 20.0, 2);
    let threshold = 0f32;
    let m = GenericMeshBuilder::new();
    let blocks = BlockStarView::new_relaying_to_field(&Sphere, block, &TransitionSide::none());
    let m = extract(&blocks, threshold, m).build();

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

#[test]
fn random_data() {
    // Just to get some small confidence that it doesn't crash
    let seed = match env::var("TEST_SEED") {
        Ok(s) => s.parse::<u64>().unwrap(),
        _ => rng().random(),
    };
    println!("Using seed {}", seed);
    let mut rng = StdRng::seed_from_u64(seed);
    let subdivisions = rng.random_range(2..30);
    let rng2 = RefCell::new(StdRng::seed_from_u64(rng.next_u64()));
    let block = Block::new([0.0, 0.0, 0.0], 10.0, subdivisions);
    let sides = random_sides(&mut rng);
    let source = |_, _, _| {
        rng2.borrow_mut().random_range(-1.0..1.0)
    };
    let m = GenericMeshBuilder::new();
    let blocks = BlockStarView::new_relaying_to_field(&source, block, &sides);
    let m = extract(&blocks, 0.5, m).build();
    println!(
        "Extracted mesh with {} tris for sides {:?}",
        m.num_tris(),
        sides
    );
}

fn random_sides(rng: &mut StdRng) -> TransitionSides {
    let mut sides = TransitionSide::none();
    for s in TransitionSide::LIST {
        if rng.random_bool(0.5) {
            let ss: TransitionSides = (*s).into();
            sides |= ss;
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
    //  for this access the density at (-1, 1, 1), which is on the regular voxels grid. In earlier versions
    // of the algorithm, this failed because the extended regular grid data was only loaded/accessible if a
    // regular cell needed it
    let subdivisions = 3;
    let size = 3.0;
    let block = Block::new([0.0, 0.0, 0.0], size, subdivisions);
    let sides = TransitionSide::LowX.into();
    let source = |x: f32, y: f32, z: f32| {
        if ((x - 0.0).abs() > f32::EPSILON)
            || ((y - 1.5).abs() > f32::EPSILON)
            || ((z - 1.0).abs() > f32::EPSILON) {
            0f32
        } else {
            1f32
        }
    };
    let m = GenericMeshBuilder::new();
    let blocks = BlockStarView::new_relaying_to_field(&source, block, &sides);
    let m = extract(&blocks, 0.5, m).build();
    println!(
        "Extracted mesh with {} tris for sides {:?}",
        m.num_tris(),
        sides
    );
}

