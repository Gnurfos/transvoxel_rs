use transvoxel::extraction::extract;
use transvoxel::structs::block::Block;
use transvoxel::structs::block_star_view::BlockStarView;
use transvoxel::structs::generic_mesh::*;
use transvoxel::traits::data_field::DataField;
use transvoxel::structs::transition_sides::TransitionSide;

struct Sphere;
impl DataField<f32, f32> for Sphere {
    fn get_data(&self, x: f32, y: f32, z: f32) -> f32 {
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
    let block = Block::new([0.0, 0.0, 0.0], 10.0, subdivisions);

    // Extract from a field
    let field = Sphere;
    let blocks = BlockStarView::new_relaying_to_field(&field, block, &TransitionSide::LowX.into());
    let mesh = GenericMeshBuilder::new();
    let mesh = extract(&blocks, THRESHOLD, mesh).build();
    println!("Extracted mesh: {:#?}", mesh);

    // Extract from a simple field function
    // let mut field = DataFieldForFn(sphere_density);
    let blocks = BlockStarView::new_relaying_to_field(&sphere_density, block, &TransitionSide::LowX.into());
    let mesh = GenericMeshBuilder::new();
    let mesh = extract(&blocks, THRESHOLD, mesh).build();
    println!("Extracted mesh: {:#?}", mesh);

    // Extract from a simple field closure
    let field = |x: f32, y: f32, z: f32| 1f32 - (x * x + y * y + z * z).sqrt() / 5f32;
    let blocks = BlockStarView::new_relaying_to_field(&field, block, &TransitionSide::LowX.into());
    let mesh = GenericMeshBuilder::new();
    let mesh = extract(&blocks, THRESHOLD, mesh).build();
    println!("Extracted mesh: {:#?}", mesh);
}
