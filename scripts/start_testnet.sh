#!/bin/sh

# build and run local testnet
cargo build --release
./target/release/xy-chain --dev
