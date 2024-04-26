# Xy Chain

Xy Chain is a Rust/substrate learning project that implements various De-Fi banking functionalities.

This project uses the [Substrate framework](https://github.com/paritytech/polkadot-sdk/tree/master/substrate).

## Introduction

This chain provides a De-Fi banking platform with the following pallets:

- Pallet-roles: Manages account roles and permissions, such as registering customers and unregistering them.
- Pallet-bank: Handles basic accounting functionalities like deposit, transfer, stake, redeem, auditor lock and unlock funds, and manager set interest rate.
- Pallet-lottery: Facilitates drawing lotteries and paying taxes, including functionalities like buying tickets and manager setting ticket prices.
- Pallet-governance: Allows governance to send extrinsics with Governance Origin and perform actions not allowed by a normal user, such as rotating authorities, force transfer, rotate treasury account, force draw lottery, force burn NFT, etc.
- Pallet-Nft: Enables the creation and trading of NFTs via Paid-On-Delivery (POD), including functionalities like requesting mint, burning, auditor approval, creating, receiving, and canceling POD.

## How To Use

You can interact with the Xy Chain using the provided scripts and functionalities.

### Prerequisite

Since the xy-chain is based on the Substrate frame, [Rust](https://www.rust-lang.org/tools/install) is required to run this chain.


### Build and Run a Localnet

To build and run a local network, use the following cargo command:

```bash
./scripts/start_testnet.sh
```


### Benchmark a Pallet
To run a benchmark, use the following command and add the pallet-name's "name", e.g. To benchmark pallet-bank:

```bash
./scripts/benchmark_pallet.sh bank
```

### Run Unit and Integration Tests
To run tests, use the following command:

```bash
cargo xy-test
```

### Run Clippy Checks
To run clippy check, use the following command:

```bash
cargo xy-clippy
```

### Build Release
To build release, use the following command:

```bash
cargo xy-build
```


### Connect with Polkadot-JS Apps Front-End

After you start the Xy Chain locally, you can interact with it using the
hosted version of the [Polkadot/Substrate
Portal](https://polkadot.js.org/apps/#/explorer?rpc=ws://localhost:9944)
front-end by connecting to the local node endpoint.
Refer to the 
[`polkadot-js/apps`](https://github.com/polkadot-js/apps) repository for guidance.

### Connect with the Xy Chain front-end

After you download and set up the [xy-chain-frontend](https://github.com/IRISxiaowang/xy-chain-frontend), you can run it with the following command:

```bash
yarn start
```
