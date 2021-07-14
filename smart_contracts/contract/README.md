# `casper-contract`

[![LOGO](../../images/CasperLabs_Logo_Horizontal_RGB.png)](https://casperlabs.io/)

[![Build Status](https://drone-auto.casperlabs.io/api/badges/CasperLabs/casper-node/status.svg?branch=master)](http://drone-auto.casperlabs.io/CasperLabs/casper-node)
[![Crates.io](https://img.shields.io/crates/v/casper-contract)](https://crates.io/crates/casper-contract)
[![Documentation](https://docs.rs/casper-contract/badge.svg)](https://docs.rs/casper-contract)
[![License](https://img.shields.io/badge/license-COSL-blue.svg)](../../LICENSE)

A library for developing CasperLabs smart contracts.

## `no_std`

It is recommended to use the library with the default features which provides a no-std environment.  Compiling a Wasm
smart contract from Rust with `no_std` enabled generally yields smaller, and hence cheaper, binaries.

For convenience, the crate provides a global allocator suitable for use in a `no_std` environment.  This can be enabled
via the `provide-allocator` feature.  **Note that using `provide-allocator` requires a nightly version of Rust**.

If you wish to work outside a `no_std` environment, then disable the default features and enable the `std` feature of
this crate.  For example:

```toml
casper-contract = { version = "1.0.0", default-features = false, features = ["std"] }
```


## License

Licensed under the [CasperLabs Open Source License (COSL)](../../LICENSE).
