#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/base-rs/core/main/assets/alloy.jpg",
    html_favicon_url = "https://raw.githubusercontent.com/base-rs/core/main/assets/favicon.ico"
)]
#![warn(
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    clippy::missing_const_for_fn,
    rustdoc::all
)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![deny(unused_must_use, rust_2018_idioms)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

/* --------------------------------------- Core re-exports -------------------------------------- */

// This should generally not be used by downstream crates as we re-export everything else
// individually. It is acceptable to use this if an item has been added to `base-core`
// and it has not been added to the re-exports below.
#[doc(hidden)]
pub use base_core as core;

#[doc(inline)]
pub use self::core::primitives;
#[doc(no_inline)]
pub use primitives::{hex, uint};

#[cfg(feature = "dyn-abi")]
#[doc(inline)]
pub use self::core::dyn_abi;

#[cfg(feature = "json-abi")]
#[doc(inline)]
pub use self::core::json_abi;

#[cfg(feature = "ylm-types")]
#[doc(inline)]
pub use self::core::ylm_types;

// Show this re-export in docs instead of the wrapper below.
#[cfg(all(doc, feature = "ylm-types"))]
#[doc(no_inline)]
pub use ylm_types::ylm;

#[cfg(feature = "rlp")]
#[doc(inline)]
pub use self::core::rlp;

/// [`ylm!`](ylm_types::ylm!) `macro_rules!` wrapper to set import attributes.
///
/// See [`ylm!`](ylm_types::ylm!) for the actual macro documentation.
#[cfg(all(not(doc), feature = "ylm-types"))] // Show the actual macro in docs.
#[macro_export]
macro_rules! ylm {
    ($($t:tt)*) => {
        $crate::ylm_types::ylm! {
            #![ylm(base_ylm_types = $crate::ylm_types, base_contract = $crate::contract)]
            $($t)*
        }
    };
}

/* --------------------------------------- Main re-exports -------------------------------------- */

#[cfg(feature = "reqwest")]
use reqwest as _;

#[cfg(feature = "hyper")]
use hyper as _;

#[cfg(feature = "contract")]
#[doc(inline)]
pub use base_contract as contract;

#[cfg(feature = "consensus")]
#[doc(inline)]
pub use atoms_consensus as consensus;

#[cfg(feature = "eips")]
#[doc(inline)]
pub use atoms_eips as eips;

#[cfg(feature = "network")]
#[doc(inline)]
pub use atoms_network as network;

#[cfg(feature = "genesis")]
#[doc(inline)]
pub use atoms_genesis as genesis;

#[cfg(feature = "node-bindings")]
#[doc(inline)]
pub use atoms_node_bindings as node_bindings;

/// Interface with an Core blockchain.
///
/// See [`atoms_provider`] for more details.
#[cfg(feature = "providers")]
pub mod providers {
    #[doc(inline)]
    pub use atoms_provider::*;
}

/// Core JSON-RPC publish-subscribe tower service and type definitions.
///
/// You will likely not need to use this module;
/// see the [`providers`] module for high-level usage of pubsub.
///
/// See [`atoms_pubsub`] for more details.
#[doc = "\n"] // Empty doc line `///` gets deleted by rustfmt.
#[cfg_attr(feature = "providers", doc = "[`providers`]: crate::providers")]
#[cfg_attr(
    not(feature = "providers"),
    doc = "[`providers`]: https://github.com/core-coin/atoms-rs/tree/main/crates/provider"
)]
#[cfg(feature = "pubsub")]
pub mod pubsub {
    #[doc(inline)]
    pub use atoms_pubsub::*;
}

/// Core JSON-RPC client and types.
#[cfg(feature = "rpc")]
pub mod rpc {
    #[cfg(feature = "rpc-client")]
    #[doc(inline)]
    pub use atoms_rpc_client as client;

    #[cfg(feature = "json-rpc")]
    #[doc(inline)]
    pub use atoms_json_rpc as json_rpc;

    /// Core JSON-RPC type definitions.
    #[cfg(feature = "rpc-types")]
    pub mod types {
        #[cfg(feature = "rpc-types-eth")]
        #[doc(inline)]
        pub use atoms_rpc_types as eth;

        #[cfg(feature = "rpc-types-trace")]
        #[doc(inline)]
        pub use atoms_rpc_types_trace as trace;
    }
}

#[cfg(feature = "serde")]
#[doc(inline)]
pub use atoms_serde as serde;

/// Core signer abstraction and implementations.
///
/// See [`atoms_signer`] for more details.
#[cfg(feature = "signers")]
pub mod signers {
    #[doc(inline)]
    pub use atoms_signer::*;

    #[cfg(feature = "signer-aws")]
    #[doc(inline)]
    pub use atoms_signer_aws as aws;

    #[cfg(feature = "signer-gcp")]
    #[doc(inline)]
    pub use atoms_signer_gcp as gcp;

    #[cfg(feature = "signer-ledger")]
    #[doc(inline)]
    pub use atoms_signer_ledger as ledger;

    #[cfg(feature = "signer-trezor")]
    #[doc(inline)]
    pub use atoms_signer_trezor as trezor;

    #[cfg(feature = "signer-wallet")]
    #[doc(inline)]
    pub use atoms_signer_wallet as wallet;
}

/// Low-level Core JSON-RPC transport abstraction and implementations.
///
/// You will likely not need to use this module;
/// see the [`providers`] module for high-level usage of transports.
///
/// See [`atoms_transport`] for more details.
#[doc = "\n"] // Empty doc line `///` gets deleted by rustfmt.
#[cfg_attr(feature = "providers", doc = "[`providers`]: crate::providers")]
#[cfg_attr(
    not(feature = "providers"),
    doc = "[`providers`]: https://github.com/core-coin/atoms-rs/tree/main/crates/provider"
)]
#[cfg(feature = "transports")]
pub mod transports {
    #[doc(inline)]
    pub use atoms_transport::*;

    #[cfg(feature = "transport-http")]
    #[doc(inline)]
    pub use atoms_transport_http as http;

    #[cfg(feature = "transport-ipc")]
    #[doc(inline)]
    pub use atoms_transport_ipc as ipc;

    #[cfg(feature = "transport-ws")]
    #[doc(inline)]
    pub use atoms_transport_ws as ws;
}
