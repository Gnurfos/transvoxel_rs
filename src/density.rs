/*!
    Density traits and storage facilities used by the extraction algorithm
*/

use std::cell::RefCell;
use num::Float;

/**
Trait that must be implemented for coordinates (x/y/z)
It mostly has to be a float with a few conversions from integers
*/
pub trait Coordinate: Float {
    
    /**
    A value of this type representing the ratio a / b
    */
    fn from_ratio(a: isize, b: usize) -> Self;

    /**
    A value of this type representing 0.5
    */
    fn half(a: isize) -> Self;
}

/**
Trait that must be implemented for a type to be used as a density by the algorithm
*/
pub trait Density: Default + Clone + Copy {

    /**
    TODO comment: gradient, coord, factor
    */
    type F: Coordinate;

    /**
    TODO comment
    */
    fn inside(&self, threshold: &Self) -> bool;


    /**
        Given:
            a, b, c gradients of the density over the 3 space directions
        Return:
            [x, y, z] a unit vector in space representing the surface normal
    */
    fn to_normal(a: &Self::F, b: &Self::F, c: &Self::F) -> [Self::F; 3];

    /**
        Returns the interpolation between a and b where threshold is crossed
        This is used to calculate world coordinates, so it must return that type
        Ex:
            a = 0, b = 5, threshold = 1
            => 0.2
    */
    fn interp(a: &Self, b: &Self, threshold: &Self) -> Self::F;

    /**
    Difference between 2 densities. Must return a Coordinate (this will be used to produce vertex normals)
    */
    fn diff(&self, other: Self) -> Self::F;

    /**
    What portion of a cell space is reserved for placing transition cells (ex/typically 0.15)
    This is part of the trait rather than a constant or parameter, because we need this value in the correct type
    */
    fn shrink_factor() -> Self::F;

}


impl Coordinate for f32 {
    fn from_ratio(a: isize, b: usize) -> Self {
        a as f32 / b as f32
    }

    fn half(a: isize) -> Self {
        0.5f32 * a as f32
    }
}

impl Density for f32 {
    
    type F = f32;

    fn inside(&self, threshold: &Self) -> bool {
        self > threshold
    }

    fn to_normal(a: &Self, b: &Self, c: &Self) -> [f32; 3] {
        let norm = (a * a + b * b + c * c).sqrt();
        if norm > f32::EPSILON {
            [-a / norm, -b / norm, -c / norm]
        } else {
            [0f32, 0f32, 0f32]
        }
    }

    fn interp(a: &Self, b: &Self, threshold: &Self) -> f32 {
        if (b - a).abs() > f32::EPSILON {
            (threshold - a) / (b - a)
        } else {
            0.5f32
        }
    }

    fn diff(&self, other: Self) -> Self::F {
        self - other
    }

    fn shrink_factor() -> Self::F {
        0.15
    }

}

/**
A source of "world" density (gives density for any world x,y,z coordinates)
*/
pub trait ScalarField<D, F> {
    /**
    Obtain the density at the given point in space
    */
    fn get_density(&self, x: F, y: F, z: F) -> D;
}

/**
ScalarField implementation for references
*/
impl<D, F, FIELD> ScalarField<D, F> for &mut FIELD
where
    FIELD: ScalarField<D, F> + ?Sized,
{
    fn get_density(&self, x: F, y: F, z: F) -> D {
        (**self).get_density(x, y, z)
    }
}

/**
Wrapper for using closures as [ScalarField]
We need the newtype wrapping because we implement ScalarField for &ScalarField too, and that would conflict without the wrapping
*/
pub struct ScalarFieldForFn<FN>(pub FN);

/**
ScalarField implementation for closures
 */
impl<D, F, FN> ScalarField<D, F> for ScalarFieldForFn<FN>
where
    FN: Fn(F, F, F) -> D,
{
    fn get_density(&self, x: F, y: F, z: F) -> D {
        self.0(x, y, z)
    }
}

/**
Wrapper for using mutable closures as [ScalarField]
We need the newtype wrapping because we implement ScalarField for &ScalarField too, and that would conflict without the wrapping
*/
pub struct ScalarFieldForFnMut<FN>(pub RefCell<FN>);

/**
ScalarField implementation for mutable closures
 */
impl<D, F, FN> ScalarField<D, F> for ScalarFieldForFnMut<FN>
where
    FN: FnMut(F, F, F) -> D,
{
    fn get_density(&self, x: F, y: F, z: F) -> D {
        self.0.borrow_mut()(x, y, z)
    }
}