use std::cell::RefCell;
use std::collections::HashMap;
use std::env;

use crate::extraction::extract;
use crate::structs::generic_mesh::*;
use crate::structs::transition_sides::*;
use bevy::math::f32;
use hamcrest2::prelude::*;
use ordered_float::OrderedFloat;
use rand::prelude::*;
use rand::rng;
use rand::Rng;
use crate::structs::block::Block;
use crate::structs::block_star_view::BlockStarView;
use crate::structs::voxel_blocks::VoxelVecBlock;
use crate::traits::data_field::DataField;



#[test]
fn count_density_calls_random() {

    let seed = match env::var("TEST_SEED") {
        Ok(s) => s.parse::<u64>().unwrap(),
        _ => rng().random(),
    };
    println!("Using seed {}", seed);
    let mut rng = StdRng::seed_from_u64(seed);

    let subdivisions = rng.random_range(2..30);

    let block = Block::new([0.0, 0.0, 0.0], 10.0, subdivisions);

    let transition_sides = random_sides(&mut rng);

    let field = RandomCountingField::new(rng);

    // Initially, no call have been made to the density function
    println!("Initial calls {}", field.calls());
    assert_that!(field.calls(), equal_to(0));

    fn one_block_calls(subs: usize) -> usize {
        let in_block_samples = (subs + 1).pow(3);
        let extensions = (subs + 1).pow(2) * 6;
        in_block_samples + extensions
    }

    let central = VoxelVecBlock::cache(&field, block);
    println!("After central block caching {}", field.calls());
    let central_calls = one_block_calls(subdivisions);
    assert_that!(field.calls(), equal_to(central_calls));

    let side_blocks = transition_sides
        .clone()
        .into_iter()
        .map(|s| (s, VoxelVecBlock::cache(&field, block.high_res_neighbour_to(s))))
        .collect::<Vec<_>>();
    let mut blocks = BlockStarView::new_simple(&central);
    let transition_sides_count = transition_sides.into_iter().count();
    println!("After ({}) side blocks caching {}", transition_sides_count, field.calls());
    let one_neighbor_calls = one_block_calls(subdivisions * 2);
    let all_neighbor_calls = transition_sides_count * one_neighbor_calls;
    assert_that!(field.calls(), equal_to(central_calls + all_neighbor_calls));

    for (side, block) in side_blocks {
        blocks = blocks.with_neighbour(block, side);
    }

    let b = GenericMeshBuilder::new();
    let _ = extract(&blocks, 0.0, b).build();
    // All blocks were accessed through caches, so no additional call has been made
    // during the extraction.
    assert_that!(field.calls(), equal_to(central_calls + all_neighbor_calls));

}

struct RandomCountingField {
    /// Counts the total number of calls, including repetitions for the same input
    calls: RefCell<usize>,
    /// Remember the values yielded, in order to be consistent and return the same, if the same inputs are queried again
    cache: RefCell<HashMap<(OrderedFloat<f32>, OrderedFloat<f32>, OrderedFloat<f32>), f32>>,
    rng: RefCell<StdRng>,
}

impl RandomCountingField {
    pub fn new(rng: StdRng) -> Self {
        Self { calls: RefCell::new(0), cache: RefCell::new(HashMap::new()), rng: RefCell::new(rng) }
    }
    fn sample(&self, _x: f32, _y: f32, _z: f32) -> f32 {
        self.rng.borrow_mut().random_range(-1.0..1.0)
    }
    pub fn calls(&self) -> usize {
        *self.calls.borrow()
    }
}

impl DataField<f32, f32> for RandomCountingField {
    fn get_data(&self, x: f32, y: f32, z: f32) -> f32 {
        *self.calls.borrow_mut() += 1;
        let mut cache = self.cache.borrow_mut();
        let density = cache.entry((OrderedFloat(x), OrderedFloat(y), OrderedFloat(z)))
            .or_insert_with(|| self.sample(x, y, z));
        *density
    }
}