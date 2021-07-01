use transvoxel::{density::*, structs::Block, voxel_source::WorldMappingVoxelSource};

use transvoxel::extraction::*;
use transvoxel::transition_sides::TransitionSide;

struct Sphere;
impl ScalarField<f32, f32> for Sphere {
    fn get_density(&self, x: f32, y: f32, z: f32) -> f32 {
        sphere_density(x, y, z)
    }
}

fn sphere_density(x: f32, y: f32, z: f32) -> f32 {
    1f32 - (x * x + y * y + z * z).sqrt() / 5f32
}

const THRESHOLD: f32 = 0f32;

fn main() {
    // Extraction parameters: world zone and subdivisions
    let subdivisions = 3;
    let block = Block::from([0.0, 0.0, 0.0], 10.0, subdivisions);

    // Extract from a [VoxelSource]
    let mut source = WorldMappingVoxelSource {
        field: &mut Sphere {},
        block: &block,
    };
    let mesh = extract(&mut source, &block, THRESHOLD, TransitionSide::LowX.into());
    println!("Extracted mesh: {:#?}", mesh);

    // Extract from a [ScalarField]
    let mut field = Sphere {};
    let mesh = extract_from_field(&mut field, &block, THRESHOLD, TransitionSide::LowX.into());
    println!("Extracted mesh: {:#?}", mesh);

    // Extract from a simple field function
    let mut field = ScalarFieldForFn(sphere_density);
    let mesh = extract_from_field(&mut field, &block, THRESHOLD, TransitionSide::LowX.into());
    println!("Extracted mesh: {:#?}", mesh);

    // Extract from a simple field closure
    let mut field =
        ScalarFieldForFn(|x: f32, y: f32, z: f32| 1f32 - (x * x + y * y + z * z).sqrt() / 5f32);
    let mesh = extract_from_field(&mut field, &block, THRESHOLD, TransitionSide::LowX.into());
    println!("Extracted mesh: {:#?}", mesh);
}
