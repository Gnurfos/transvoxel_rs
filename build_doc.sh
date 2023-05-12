#!/bin/bash
set -e
cargo test
cargo readme > README.md
cargo doc --no-deps --document-private-items --open
