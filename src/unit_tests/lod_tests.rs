/**
 * ######################################################################################
 * # Tests to see if the same data can be extracted in multiple Level Of Detail safely
 * ######################################################################################
 */

/**
 * Generates noise to check the surface genration against
 * */
struct NoiseContainer {
    pub(crate) buffer: Vec<Vec<Vec<f32>>>,
}

impl NoiseContainer {
    fn size(&self) -> usize {
        self.buffer.len()
    }

    fn block(dimensions: usize) -> NoiseContainer {
        let mut buffer = vec![vec![vec![0.; dimensions]; dimensions]; dimensions];
        let normalizer = dimensions as f64;
        /* Because the seed for noises are not working, an offset is used to simulate actual randomness */
        use rand::{Rng, SeedableRng}; 
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as u64;
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed as u64);
        let random_offset: Vec<f64> = (0..3).map(|_| rng.gen_range(0.0..1000.0)).collect();

        use noise::HybridMulti;
        use noise::NoiseFn;
        use noise::OpenSimplex;
        let surface_noise = HybridMulti::<OpenSimplex>::new(seed as u32);
        /* 1..dimensions-1 range is to have an enclosed container */
        for x in 1..dimensions - 1 {
            for y in 1..dimensions - 1 {
                for z in 1..dimensions - 1 {
                    buffer[x][y][z] = {
                        let normalized_x = random_offset[0] + x as f64 / normalizer;
                        let normalized_y = random_offset[1] + y as f64 / normalizer;
                        let normalized_z = random_offset[2] + z as f64 / normalizer;
                        surface_noise.get([normalized_x, normalized_y, normalized_z]) as f32
                    };
                }
            }
        }
        NoiseContainer { buffer }
    }
}

use crate::density::ScalarField;
impl ScalarField<f32, f32> for &NoiseContainer {
    fn get_density(&self, x: f32, y: f32, z: f32) -> f32 {
        let ix = x.round() as i32;
        let iy = y.round() as i32;
        let iz = z.round() as i32;
        if (ix < 0 || ix >= self.size() as i32)
            || (iy < 0 || iy >= self.size() as i32)
            || (iz < 0 || iz >= self.size() as i32)
        {
            return 0.;
        }

        self.buffer[ix as usize][iy as usize][iz as usize]
    }
}

use crate::structs::Block;
use crate::extraction::extract_from_field;
use crate::transition_sides::no_side;
#[test] /* - For checking if extracting the surface for different resolutions won't crash the app */
fn extract_field_in_multiple_resolution() {
	let noise = NoiseContainer::block(64);
	for resolution in 4..noise.size() {
		let block = Block::from([0.,0.,0.], noise.size() as f32, resolution);
		let _mesh = extract_from_field(&noise, &block, 0.0, no_side());
	}
}