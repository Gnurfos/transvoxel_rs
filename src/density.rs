/*!
    Density traits and storage facilities used by the extraction algorithm
*/

use num::Float;

/**
Trait that must be implemented for a type to be used as a density by the algorithm
For the moment, it has to be a [Float]
*/
pub trait Density: Float {
    /**
        Given:
            a, b, c gradients of the density over the 3 space directions
        Return:
            [x, y, z] a unit vector in space representing the surface normal
    */
    fn to_normal(a: &Self, b: &Self, c: &Self) -> [f32; 3];

    /**
        Returns the interpolation between a and b where threshold is crossed
        Ex:
            a = 0, b = 5, threshold = 1
            => 0.2
    */
    fn interp(a: &Self, b: &Self, threshold: &Self) -> Self;

    /**
        Convert to a f32, for scaling world positions
    */
    fn as_f32(&self) -> f32;
}

impl Density for f32 {
    fn interp(a: &Self, b: &Self, threshold: &Self) -> f32 {
        if (b - a).abs() > f32::EPSILON {
            (threshold - a) / (b - a)
        } else {
            0.5f32
        }
    }
    fn as_f32(&self) -> f32 {
        *self
    }

    fn to_normal(a: &Self, b: &Self, c: &Self) -> [f32; 3] {
        let norm = (a * a + b * b + c * c).sqrt();
        if norm > f32::EPSILON {
            [-a / norm, -b / norm, -c / norm]
        } else {
            [0f32, 0f32, 0f32]
        }
    }
}

/**
A source of "world" density (gives density for any world x,y,z coordinates)
*/
pub trait ScalarField<D> {
    /**
    Obtain the density at the given point in space
    */
    fn get_density(&mut self, x: f32, y: f32, z: f32) -> D;
}

/**
ScalarField implementation for references
*/
impl<D, F> ScalarField<D> for &mut F
where
    F: ScalarField<D> + ?Sized,
{
    fn get_density(&mut self, x: f32, y: f32, z: f32) -> D {
        (**self).get_density(x, y, z)
    }
}

/**
Wrapper for using closures as [ScalarField]
We need the newtype wrapping because we implement ScalarField for &ScalarField too, and that would conflict without the wrapping
*/
pub struct ScalarFieldForFn<F>(pub F);

/**
ScalarField implementation for closures
 */
impl<F, D> ScalarField<D> for ScalarFieldForFn<F>
where
    F: FnMut(f32, f32, f32) -> D,
{
    fn get_density(&mut self, x: f32, y: f32, z: f32) -> D {
        self.0(x, y, z)
    }
}
