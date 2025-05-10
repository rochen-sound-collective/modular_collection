#!/bin/bash
cargo xtask bundle modular_euclidian --release
cp 'target/bundled/modular_euclidian.clap' ~/.clap/

