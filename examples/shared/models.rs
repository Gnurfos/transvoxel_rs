use std::collections::HashMap;

use noise::{Fbm, NoiseFn, Perlin};
use std::slice::Iter;
use transvoxel::density::ScalarField;

#[derive(PartialEq, Debug, Copy, Clone, Hash, Eq)]
pub enum Model {
    Sphere,
    Quadrant,
    Plane,
    Wave,
    Noise,
}

pub fn models_map() -> HashMap<Model, Box<dyn ScalarField<f32, f32>>> {
    let mut fields: HashMap<Model, Box<dyn ScalarField<f32, f32>>> = HashMap::new();
    fields.insert(
        Model::Sphere,
        Box::new(Sphere {
            cx: 5f32,
            cy: 5f32,
            cz: 5f32,
            r: 2f32,
        }),
    );
    fields.insert(
        Model::Quadrant,
        Box::new(Sphere {
            cx: 0f32,
            cy: 0f32,
            cz: 0f32,
            r: 6f32,
        }),
    );
    fields.insert(Model::Plane, Box::new(ObliquePlane {}));
    fields.insert(Model::Wave, Box::new(Wave {}));
    fields.insert(Model::Noise, Box::new(Noise::new()));
    return fields;
}

pub const THRESHOLD: f32 = 0.;

impl Model {
    pub fn iterator() -> Iter<'static, Model> {
        static MODELS: [Model; 5] = [
            Model::Sphere,
            Model::Quadrant,
            Model::Plane,
            Model::Wave,
            Model::Noise,
        ];
        MODELS.iter()
    }
}

struct Sphere {
    pub cx: f32,
    pub cy: f32,
    pub cz: f32,
    pub r: f32,
}

impl ScalarField<f32, f32> for Sphere {
    fn get_density(&self, x: f32, y: f32, z: f32) -> f32 {
        let distance_from_center = ((x - self.cx) * (x - self.cx)
            + (y - self.cy) * (y - self.cy)
            + (z - self.cz) * (z - self.cz))
            .sqrt();
        let d = 1f32 - distance_from_center / self.r;
        d
    }
}

struct ObliquePlane {}
impl ScalarField<f32, f32> for ObliquePlane {
    #[allow(unused_variables)]
    fn get_density(&self, x: f32, y: f32, z: f32) -> f32 {
        2f32 + z - 2f32 * y
    }
}

struct Wave {}
impl ScalarField<f32, f32> for Wave {
    fn get_density(&self, x: f32, y: f32, z: f32) -> f32 {
        2.0 * ((x * 1.0).sin() + 0.5 * (z * 0.5).cos()) + 5.0 - y
    }
}

struct Noise {
    f: Box<dyn NoiseFn<f64, 3>>,
}
impl Noise {
    pub fn new() -> Self {
        Self {
            f: Box::new(Fbm::<Perlin>::new(0)),
        }
    }
}
impl ScalarField<f32, f32> for Noise {
    fn get_density(&self, x: f32, y: f32, z: f32) -> f32 {
        let distrub = self.f.get([x as f64, y as f64, z as f64]) as f32;
        2f32 - 2f32 * (y - 3.0 - 3.0 * distrub)
    }
}
