#! /bin/bash
cargo build --release
rm -f ./compiled_pack/occupier_r_linux
mv ./target/release/occupier_r ./compiled_pack/occupier_r_linux