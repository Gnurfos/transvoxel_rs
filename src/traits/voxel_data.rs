/*!
   Traits for defining voxel data and density
*/

use num::Float;


/// Trait that a type must implement to be used as voxel data (return values of a [DataField] or [VoxelBlock]).
/// Anything can be stored, but the type has to provide a float density for the algorithm to work with.
/// Any additional data stored can be used by a custom [MeshBuilder].
///
/// [MeshBuilder]: crate::traits::mesh_builder::MeshBuilder
/// [DataField]: crate::traits::data_field::DataField
/// [VoxelBlock]: crate::traits::voxel_block::VoxelBlock
pub trait VoxelData {
    /// The type that acts as density
    type Density: Density;
    /// How to get the density from a given data object
    fn density(&self) -> Self::Density;
}

/// A [Density] by itself can act as [VoxelData] (no need to attach additional data).
impl<F: Density> VoxelData for F {
    type Density = Self;
    fn density(&self) -> Self::Density {
        *self
    }
}

/**
Trait that must be implemented for a type to be used as a density by the algorithm
*/
pub trait Density: Default + Clone + Copy + Float {
    /// How to determine whether a point with a given density is inside or outside the mesh
    fn inside(&self, threshold: &Self) -> bool {
        self > threshold
    }

    /// Epsilon value (used for float comparisons)
    const EPSILON: Self;
    /// Value for 0.5
    const HALF: Self;
    /// Value for 0
    const ZERO: Self;

    /// Interpolate to determine where between A and B the threshold is crossed
    fn interpolate(a: Self, b: Self, threshold: Self) -> Self {
        if (b - a).abs() > Self::EPSILON {
            (threshold - a) / (b - a)
        } else {
            Self::HALF
        }
    }

    /// Subtraction
    fn diff(&self, other: Self) -> Self {
        *self - other
    }

    /// Convert 3 directional gradients of the density to a vector orthogonal to the surface, and pointing out
    fn gradients_to_normal(x_gradient: Self, y_gradient: Self, z_gradient: Self) -> [Self; 3] {
        let norm =
            (x_gradient * x_gradient + y_gradient * y_gradient + z_gradient * z_gradient).sqrt();
        if norm > Self::EPSILON {
            [-x_gradient / norm, -y_gradient / norm, -z_gradient / norm]
        } else {
            [Self::ZERO, Self::ZERO, Self::ZERO]
        }
    }
}


macro_rules! float_impl_density {
    ($T:ident) => {
        impl Density for $T {
            const EPSILON: Self = $T::EPSILON;
            const HALF: Self = 0.5;
            const ZERO: Self = 0.0;
        }
    };
}

float_impl_density!(f32);
float_impl_density!(f64);