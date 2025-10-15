#![warn(missing_docs)]
/*!

This is an implementation of Eric Lengyel's Transvoxel Algorithm in Rust

Credits: Eric Lengyel's Transvoxel Algorithm. <https://transvoxel.org/>

# Brief description of the problem
When extracting meshes with Marching Cubes for adjacent blocks at different level of detail independently, the meshes generally do not match at the blocks' junctions, inducing visible holes:
![gap-solid](https://gnurfos.github.io/transvoxel_rs/doc-images/gap-solid.png)
![gap-wireframe](https://gnurfos.github.io/transvoxel_rs/doc-images/gap-wireframe.png)
The Transvoxel Algorithm allows generating a mesh for a block *mostly* with marching cubes, but with increased detail on one or several external faces of the block, in order to match with neighbouring blocks' meshes:
![fixed-wireframe](https://gnurfos.github.io/transvoxel_rs/doc-images/fixed-wireframe.png)

Eric Lengyel's [website](https://transvoxel.org/) describes this better and in more details.

# Scope
This library only provides functions for extracting a mesh for a block, independently of other blocks.
To implement a fully consistent dynamic level-of-detail system, you will also probably need to:
 * decide which blocks you need to render and generate meshes for, and at which resolution (typically depending on the camera position and/or orientation)
 * track yourself constraints:
   * two rendered adjacent blocks can only either have the same resolution, or one have double the resolution of the other
   * in that second case, the low resolution block must also be rendered with a transition face in the direction of the high resolution block

Currently, it is not possible to "flip" a transition face status on a block, without re-extracting a new mesh for the block. Which means changing the resolution for one block can cascade through constraints to re-generating a few other blocks as well

# Basic usage
Either try calling one of the functions in [extraction], or follow the example below:
```rust
// The first thing you need is a density provider. You can implement a [DataField] for that
// but a simpler way, if you are just experimenting, is to use a function:

fn sphere_density(x: f32, y: f32, z: f32) -> f32 {
    1f32 - (x * x + y * y + z * z).sqrt() / 5f32
}

// Going along with your density function, you need a threshold value for your density:
// This is the value for which the surface will be generated. You can typically choose 0.
// Values over the threshold are considered inside the volume, and values under the threshold
// outside the volume. In our case, we will have a density of 0 on a sphere centered on the
// world center, of radius 5.
let threshold = 0f32;

// Then you need to decide for which region of the world you want to generate the mesh, and how
// many subdivisions should be used (the "resolution"). You also need to tell which sides of the
// block need to be transition (double-resolution) faces. We use `no_side` here for simplicity,
// and will get just a regular Marching Cubes extraction, but the Transvoxel transitions can be
// obtained simply by providing some sides instead (that is shown a bit later):
use transvoxel::prelude::*;
let subdivisions = 10;
let block = Block::new([0.0, 0.0, 0.0], 10.0, subdivisions);
let transition_sides = TransitionSide::none();
// Finally, you can run the mesh extraction:
let builder = GenericMeshBuilder::new();
let builder = extract_from_field(
    &sphere_density,
    FieldCaching::CacheNothing,
    block,
    transition_sides,
    threshold,
    builder
);
let mesh = builder.build();
assert!(mesh.tris().len() == 103);
// Extracting with some transition faces results in a slightly more complex mesh:
use TransitionSide::LowX;
let builder = GenericMeshBuilder::new();
let builder = extract_from_field(
    &sphere_density,
    FieldCaching::CacheNothing,
    block,
    LowX.into(),
    threshold,
    builder
);
let mesh = builder.build();
assert!(mesh.tris().len() == 131);
// Unless, of course, the surface does not cross that face:
use TransitionSide::HighZ;
let builder = GenericMeshBuilder::new();
let builder = extract_from_field(
    &sphere_density,
    FieldCaching::CacheNothing,
    block,
    HighZ.into(),
    threshold,
    builder
);
let mesh = builder.build();
assert!(mesh.tris().len() == 103);
```

# How to use the resulting mesh
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
(the first triangle uses vertices 0 1 2, and the second triangle vertices 0 2 3).
This is what you get when you use the [GenericMeshBuilder], but you can and probably should
use your own [MeshBuilder]. If you're using bevy, we have an implementation in our examples code.


# How to request transition sides
```rust
use transvoxel::structs::transition_sides::TransitionSide;

// If you don't hardcode sides like in the example above, you can build a set of sides incrementally:
// They use the FlagSet crate
let mut sides = TransitionSide::none();
sides |= TransitionSide::LowX;
sides |= TransitionSide::HighY;

assert!(sides.contains(TransitionSide::LowX));
assert!(!sides.contains(TransitionSide::HighX));
```

# Realistic and efficient caching of the density field
The examples above do not perform any caching, for simplicity's sake.
Which means that the density function will be called potentially (probably)
several (many) times for the same voxel position. This is probably not suitable
for a real use case, especially if your density function is not cheap.

The idiomatic way of handling that is to call the slightly more advanced [extract] function,
and to provide it a [BlockStarView].

Say you want to extract for these 2 blocks:
```
use transvoxel::prelude::*;
let b1 = Block::new([0.0, 0.0, 0.0], 10.0, 16);
let b2 = Block::new([10.0, 0.0, 0.0], 10.0, 32); // Or b2 = b1.high_res_neighbour_to(HighX);
```
There will be 2 calls to `extract`, so caching has to be done externally, by hand.
We provide [VoxelVecBlock] for that, but if this way of storing things is not suitable, you
can implement your own [VoxelBlock] to do something similar.
```
# use transvoxel::prelude::*;
let density_function = |x, y, z| z;
let threshold = 0.0;
# let b1 = Block::new([0.0, 0.0, 0.0], 10.0, 16);
# let b2 = Block::new([10.0, 0.0, 0.0], 10.0, 32);
// Cache each block separately
let cached_b1 = VoxelVecBlock::cache(&density_function, b1);
let cached_b2 = VoxelVecBlock::cache(&density_function, b2);
// Now just use views into the cached blocks, to reuse data across `extract` calls:
// We use `&VoxelVecBlock` here. You can use anything that implements [VoxelBlock]
type View<'a> = BlockStarView<f32, f32, &'a VoxelVecBlock<f32, f32>, &'a VoxelVecBlock<f32, f32>>;
let mesh1 = extract(
    &View::new_simple(&cached_b1).with_neighbour(&cached_b2, TransitionSide::HighX),
    threshold,
    GenericMeshBuilder::new()
).build();
let mesh2 = extract(
    &View::new_simple(&cached_b2),
    threshold,
    GenericMeshBuilder::new()
).build();
```


[Algorithm]: implementation::algorithm
[Float]: num::Float
[MeshBuilder]: traits::mesh_builder::MeshBuilder
[GenericMeshBuilder]: structs::generic_mesh::GenericMeshBuilder
[BlockStarView]: structs::block_star_view::BlockStarView
[VoxelVecBlock]: structs::voxel_blocks::VoxelVecBlock
[VoxelBlock]: traits::voxel_block::VoxelBlock
[extract]: extraction::extract
*/
#![allow(rustdoc::private_intra_doc_links)]

#[cfg(test)]
mod unit_tests;

pub mod prelude;
pub mod traits;
pub mod structs;

pub mod extraction;

mod implementation;

/// These functions are only made public for some of our tests/examples. A regular use should not need them.
pub mod private {
    pub use super::implementation::voxel_coordinates::high_res_voxel_index_to_position as _internal_high_res_voxel_index_to_position;
}
