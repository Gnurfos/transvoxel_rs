/*!
   Traits for world coordinates
*/

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
    A value of this type representing a / 2
    */
    fn half(a: isize) -> Self;

    /**
    What portion of a cell space is reserved for placing transition cells (ex/typically 0.15)
    This is part of the trait rather than a constant or parameter, because we need this value in the correct type
    */
    fn shrink_factor() -> Self;
}


macro_rules! float_impl_coordinate {
    ($T:ident) => {
        impl Coordinate for $T {
            fn from_ratio(a: isize, b: usize) -> Self {
                (a as $T) / (b as $T)
            }

            fn half(a: isize) -> Self {
                0.5 * (a as $T)
            }

            fn shrink_factor() -> Self {
                0.15
            }
        }
    };
}

float_impl_coordinate!(f32);
float_impl_coordinate!(f64);
