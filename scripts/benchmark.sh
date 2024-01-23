#!/bin/sh

# usage: ./scripts/benchmark.sh palletname

cargo build --release --features runtime-benchmarks
# execute the benchmark for $palletame
./target/release/node-template benchmark pallet \
    --extrinsic '*' \
    --pallet pallet_$1 \
    --output ./pallets/pallet-$1/src/weights.rs \
    --steps=20 \
    --repeat=20 \
    --template=weight-template.hbs
