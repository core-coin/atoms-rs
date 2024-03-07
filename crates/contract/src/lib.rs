#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/alloy-rs/core/main/assets/alloy.jpg",
    html_favicon_url = "https://raw.githubusercontent.com/alloy-rs/core/main/assets/favicon.ico"
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

// todo: re-enable when alloy-sol-macro is adjusted
//#[cfg(test)]
//extern crate self as alloy_contract;

mod error;
pub use error::*;

mod interface;
pub use interface::*;

mod instance;
pub use instance::*;

mod call;
pub use call::*;

// Not public API.
// NOTE: please avoid changing the API of this module due to its use in the `sol!` macro.
#[doc(hidden)]
pub mod private {
    pub use alloy_providers::Provider;
}
