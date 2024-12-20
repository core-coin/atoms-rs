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

mod error;
pub use error::{Error, Result, UnsupportedSignerOperation};

mod signer;
pub use signer::{Signer, SignerSync};

pub mod utils;

pub use base_primitives::Signature;

/// Utility to get and set the network ID on a transaction and the resulting signature within a
/// signer's `sign_transaction`.
#[macro_export]
macro_rules! sign_transaction_with_network_id {
    // async (
    //    signer: impl Signer,
    //    tx: &mut impl SignableTransaction<Signature>,
    //    sign: lazy Signature,
    // )
    ($signer:expr, $tx:expr, $sign:expr) => {{
        if let network_id = $signer.network_id() {
            if !$tx.set_chain_id_checked(network_id) {
                return Err(atoms_signer::Error::TransactionNetworkIdMismatch {
                    signer: network_id,
                    // we can only end up here if the tx has a network id
                    tx: $tx.chain_id(),
                });
            }
        }

        let mut sig = $sign.map_err(atoms_signer::Error::other)?;

        Ok(sig)
    }};
}
