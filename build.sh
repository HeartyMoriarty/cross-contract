#!/bin/bash
cd ./simple-bank
cargo test
cd ../token-contract
cargo test
cd ../
# cargo build --all --target wasm32-unknown-unknown --release
# cp target/wasm32-unknown-unknown/release/*.wasm ./res/ 
# cargo test