# Crate transvoxel
Current version: 0.2.0

![Maintenance](https://img.shields.io/badge/maintenance-experimental-blue.svg)

(the following is generated from the crate's rustdoc. reading it on [docs.rs](https://docs.rs/transvoxel) will probably be a better experience)

This is an implementation of Eric Lengyel's Transvoxel Algorithm in Rust

Credits: Eric Lengyel's Transvoxel Algorithm. <https://transvoxel.org/>

## Brief description of the problem
When extracting meshes with Marching Cubes for adjacent blocks at different level of detail independently, the meshes generally do not match at the blocks' junctions, inducing visible holes:
![gap-solid](https://gnurfos.github.io/transvoxel_rs/doc-images/gap-solid.png)
![gap-wireframe](https://gnurfos.github.io/transvoxel_rs/doc-images/gap-wireframe.png)
The Transvoxel Algorithm allows generating a mesh for a block *mostly* with marching cubes, but with increased detail on one or several external faces of the block, in order to match with neighbouring blocks' meshes:
![fixed-wireframe](https://gnurfos.github.io/transvoxel_rs/doc-images/fixed-wireframe.png)

Eric Lengyel's [website](https://transvoxel.org/) describes this better and in more details.

## Scope
This library only provides functions for extracting a mesh for a block, independently of other blocks.
To implement a fully consistent dynamic level-of-detail system, you will also probably need to:
 * decide which blocks you need to render and generate meshes for, and at which resolution (typically depending on the camera position and/or orientation)
 * track yourself constraints:
   * two rendered adjacent blocks can only either have the same resolution, or one have double the resolution of the other
   * in that second case, the low resolution block must also be rendered with a transition face in the direction of the high resolution block
Currently, it is not possible to "flip" a transition face status on a block, without re-extracting a new mesh for the block. Which means changing the resolution for one block can cascade through constraints to re-generating a few other blocks as well

## Basic usage
Either try calling one of the functions in [extraction], or follow the example below:
```rust
// The first thing you need is a density provider. You can implement a ScalarField for that
// but a simpler way, if you are just experimenting, is to use a function:
use transvoxel::density::ScalarFieldForFn;

fn sphere_density(x: f32, y: f32, z: f32) -> f32 {
    1f32 - (x * x + y * y + z * z).sqrt() / 5f32
}

let mut field = ScalarFieldForFn(sphere_density);

// Going along with your density function, you need a threshold value for your density:
// This is the value for which the surface will be generated. You can typically choose 0.
// Values over the threshold are considered inside the volume, and values under the threshold
// outside of the volume. In our case, we will have a density of 0 on a sphere centered on the
// world center, of radius 5.
let threshold = 0f32;

// Then you need to decide for which region of the world you want to generate the mesh, and how
// many subdivisions should be used (the "resolution"). You also need to tell which sides of the
// block need to be transition (double-resolution) faces. We use `no_side` here for simplicity,
// and will get just a regular Marching Cubes extraction, but the Transvoxel transitions can be
// obtained simply by providing some sides instead (that is shown a bit later):
use transvoxel::structs::Block;
use transvoxel::transition_sides::no_side;
let subdivisions = 10;
let block = Block::from([0.0, 0.0, 0.0], 10.0, subdivisions);
let transition_sides = no_side();

// Finally, you can run the mesh extraction:
use transvoxel::extraction::extract_from_field;
let mesh = extract_from_field(&mut field, &block, threshold, transition_sides);
assert!(mesh.tris().len() == 103);

// Extracting with some transition faces results in a slightly more complex mesh:
use transvoxel::transition_sides::TransitionSide::LowX;
let mesh = extract_from_field(&mut field, &block, threshold, LowX.into());
assert!(mesh.tris().len() == 131);

// Unless, of course, the surface does not cross that face:
use transvoxel::transition_sides::TransitionSide::HighZ;
let mesh = extract_from_field(&mut field, &block, threshold, HighZ.into());
assert!(mesh.tris().len() == 103);
```

## How to use the resulting mesh
A mesh for a simple square looks like this:
```ron
Extracted mesh: Mesh {
    positions: [
        10.0,
        5.0,
        0.0,
        0.0,
        5.0,
        0.0,
        0.0,
        5.0,
        10.0,
        10.0,
        5.0,
        10.0,
    ],
    normals: [
        -0.0,
        1.0,
        -0.0,
        -0.0,
        1.0,
        -0.0,
        -0.0,
        1.0,
        -0.0,
        -0.0,
        1.0,
        -0.0,
    ],
    triangle_indices: [
        0,
        1,
        2,
        0,
        2,
        3,
    ],
}
```
It is made of 4 vertices, arranged in 2 triangles.
The first vertex is at position x=10.0, y=5.0, z=0.0 (the first 3 floats in position).
As the first in the list, it's index is 0, and we can see it is used in the 2 triangles
(the first triangle uses vertices 0 1 2, and the second triangle vertices 0 2 3)

If you need to use the mesh in [Bevy](https://bevyengine.org/), you can enable feature `bevy_mesh` and use functions in [bevy_mesh]

[bevy_mesh]: crate::bevy_mesh

## How to request transition sides
```rust
use transvoxel::transition_sides::{TransitionSide, no_side};

// If you don't hardcode sides like in the example above, you can build a set of sides incrementally:
// They use the FlagSet crate
let mut sides = no_side();
sides |= TransitionSide::LowX;
sides |= TransitionSide::HighY;

assert!(sides.contains(TransitionSide::LowX));
assert!(!sides.contains(TransitionSide::HighX));
```

## Limitations / possible improvements
 * Provide a way to extract without normals, or with face normals, which would be much faster
 * Output/Input positions/normals are only f32. It should be feasible easily to extend that to f64
 * [Density] is limited to [Float] at the moment (only implemented for f32 and could be easily extended to f64). Some thinking would be needed for allowing more types, regarding interactions with gradients and interpolation of coordinates
 * Voxel densities caching is sub-optimal: probably only in the case of an empty block will densities be queried only once per voxel. In non-empty blocks, densities are very likely to be queried several times for some voxels
 * Algorithm improvements. See [Algorithm]

[Algorithm]: crate::implementation::algorithm
[Density]: crate::density::Density
[Float]: num::Float


## License: MIT OR Apache-2.0

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
