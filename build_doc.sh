#!/bin/bash
cargo test
cargo readme > README.md
cargo doc --features bevy_mesh --no-deps --document-private-items --open
