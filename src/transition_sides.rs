/*!
Enum defining the 6 sides of a block that can be extracted at double resolution
*/

use flagset::{flags, FlagSet};

flags! {
    /**
    A block face for which the algorithm should generate "transition" cells
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

/** A set of several [TransitionSide]
Check the `flagset` crate for more details if needed.
```
# use transvoxel::transition_sides::{*, TransitionSide::*};
// You can specify "no side" (empty set)
let nothing = no_side();
let nothing_either: TransitionSides = None.into();

// You can hardcode one side
let one_side: TransitionSides = LowX.into();

// Or build a set incrementally
let mut sides = no_side();
sides |= TransitionSide::LowX;
sides |= TransitionSide::HighY;

assert!(sides.contains(TransitionSide::LowX));
assert!(!sides.contains(TransitionSide::HighX));
```
*/
pub type TransitionSides = FlagSet<TransitionSide>;

/// Empty set of sides
pub fn no_side() -> TransitionSides {
    FlagSet::<TransitionSide>::default()
}
