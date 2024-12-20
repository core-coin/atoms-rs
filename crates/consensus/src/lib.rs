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
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod constants;

mod header;
pub use header::{Header, EMPTY_OMMER_ROOT_HASH, EMPTY_ROOT_HASH};

mod receipt;
pub use receipt::{AnyReceiptEnvelope, Receipt, ReceiptWithBloom, TxReceipt};

mod transaction;
pub use transaction::{SignableTransaction, Transaction, TxLegacy, TypedTransaction};

#[cfg(feature = "kzg")]
pub use transaction::BlobTransactionValidationError;

#[cfg(feature = "kzg")]
pub use atoms_eips::eip4844::env_settings::EnvKzgSettings;

mod sealed;
pub use sealed::{Sealable, Sealed};

mod signed;
pub use signed::Signed;
