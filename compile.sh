#! /bin/bash
machine_arch=`arch`  
cargo fix --lib -p occupier_r --allow-dirty
rustup target add x86_64-unknown-linux-gnu
rustup target add aarch64-unknown-linux-gnu
cargo build --release --target aarch64-unknown-linux-gnu
cargo build --release --target x86_64-unknown-linux-gnu  