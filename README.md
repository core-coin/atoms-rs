# Alloy

Alloy connects applications to blockchains.

Alloy is a rewrite of [`ethers-rs`] from the ground up, with exciting new
features, high performance, and excellent docs.

[`ethers-rs`] will continue to be maintained until we have achieved
feature-parity in Alloy. No action is currently needed from devs.

[![Telegram chat][telegram-badge]][telegram-url]

[`ethers-rs`]: https://github.com/gakonst/ethers-rs
[telegram-badge]: https://img.shields.io/endpoint?color=neon&style=for-the-badge&url=https%3A%2F%2Ftg.sumanjay.workers.dev%2Fethers_rs
[telegram-url]: https://t.me/ethers_rs

## Overview

This repository contains the following crates:

- [`alloy-json-rpc`] - Core data types for JSON-RPC 2.0 clients.
- [`alloy-transports`] - Transport implementations for JSON-RPC 2.0 clients.
- [`alloy-networks`] - Network abstraction for RPC types. Allows capturing
  different RPC param and response types on a per-network basis.
- [`alloy-providers`] - A client trait for interacting with Ethereum-like RPC
  endpoints. Abstract over `alloy_networks::Network`, which allows capturing
  different RPC types on a per-network basis.

[`alloy-json-rpc`]: ./crates/json-rpc
[`alloy-transports`]: ./crates/transports
[`alloy-networks`]: ./crates/networks
[`alloy-providers`]: ./crates/providers

## Supported Rust Versions

<!--
When updating this, also update:
- .clippy.toml
- Cargo.toml
- .github/workflows/ci.yml
-->

Alloy will keep a rolling MSRV (minimum supported rust version) policy of **at
least** 6 months. When increasing the MSRV, the new Rust version must have been
released at least six months ago. The current MSRV is 1.68.

Note that the MSRV is not increased automatically, and only as part of a minor
release.

## Contributing

Thanks for your help improving the project! We are so happy to have you! We have
[a contributing guide](./CONTRIBUTING.md) to help you get involved in the
Alloy project.

Pull requests will not be merged unless CI passes, so please ensure that your
contribution follows the linting rules and passes clippy.

## Note on `no_std`

Because these crates are primarily json-rpc focused, we do not intend to support
`no_std` for them at this time.

## Credits

None of these crates would have been possible without the great work done in:

- [`ethers.js`](https://github.com/ethers-io/ethers.js/)
- [`rust-web3`](https://github.com/tomusdrw/rust-web3/)
- [`ruint`](https://github.com/recmo/uint)
- [`ethabi`](https://github.com/rust-ethereum/ethabi)
- [`ethcontract-rs`](https://github.com/gnosis/ethcontract-rs/)
- [`guac_rs`](https://github.com/althea-net/guac_rs/)

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in these crates by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.
</sub>
