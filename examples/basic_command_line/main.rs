use transvoxel::generic_mesh::*;
use transvoxel::voxel_source::*;

use transvoxel::extraction::*;
use transvoxel::transition_sides::TransitionSide;

struct Sphere;
impl DataField<f32, f32> for Sphere {
    fn get_data(&mut self, x: f32, y: f32, z: f32) -> f32 {
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
    let source = WorldMappingVoxelSource {
        field: Sphere {},
        block: &block,
    };
    let mesh = GenericMeshBuilder::new();
    let mesh = extract(source, &block, THRESHOLD, TransitionSide::LowX.into(), mesh).build();
    println!("Extracted mesh: {:#?}", mesh);

    // Extract from a [ScalarField]
    let field = Sphere {};
    let mesh = GenericMeshBuilder::new();
    let mesh =
        extract_from_field(field, &block, THRESHOLD, TransitionSide::LowX.into(), mesh).build();
    println!("Extracted mesh: {:#?}", mesh);

    // Extract from a simple field function
    // let mut field = DataFieldForFn(sphere_density);
    let mesh = GenericMeshBuilder::new();
    let mesh = extract_from_field(
        &sphere_density,
        &block,
        THRESHOLD,
        TransitionSide::LowX.into(),
        mesh,
    )
    .build();
    println!("Extracted mesh: {:#?}", mesh);

    // Extract from a simple field closure
    let field = |x: f32, y: f32, z: f32| 1f32 - (x * x + y * y + z * z).sqrt() / 5f32;
    let mesh = GenericMeshBuilder::new();
    let mesh =
        extract_from_field(&field, &block, THRESHOLD, TransitionSide::LowX.into(), mesh).build();
    println!("Extracted mesh: {:#?}", mesh);
}
