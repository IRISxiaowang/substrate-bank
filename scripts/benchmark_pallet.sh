#!/bin/sh

# usage: ./scripts/benchmark_pallet.sh palletname

cargo build --release --features runtime-benchmarks
# execute the benchmark for $palletame
./target/release/xy-chain benchmark pallet \
    --extrinsic '*' \
    --pallet pallet_$1 \
    --output ./pallets/pallet-$1/src/weights.rs \
    --steps=20 \
    --repeat=20 \
    --template=weight-template.hbs
