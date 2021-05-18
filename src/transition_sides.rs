/*!
Enum defining the 6 sides of a block that can be extracted at double resolution
*/

use flagset::{flags, FlagSet};

flags! {
    /**
    Block faces for which the algorithm should generate "transition" cells
    (because the neighbouring block on that face is double-resolution of this block)
    */
    #[allow(missing_docs)]
    pub enum TransitionSide: u8 {
        LowX,
        HighX,
        LowY,
        HighY,
        LowZ,
        HighZ,
    }
}

/// Set of several sides
pub type TransitionSides = FlagSet<TransitionSide>;

/// Empty set of sides
pub fn no_side() -> TransitionSides {
    FlagSet::<TransitionSide>::default()
}
