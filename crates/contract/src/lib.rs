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

#[cfg(test)]
extern crate self as base_contract;

mod eth_call;
pub use eth_call::{CallDecoder, XcbCall};

mod error;
pub use error::*;

mod event;
pub use event::{Event, EventPoller};

#[cfg(feature = "pubsub")]
pub use event::subscription::EventSubscription;

mod interface;
pub use interface::*;

mod instance;
pub use instance::*;

mod call;
pub use call::*;

// Not public API.
// NOTE: please avoid changing the API of this module due to its use in the `ylm!` macro.
#[doc(hidden)]
pub mod private {
    pub use atoms_network::{Ethereum, Network};
    pub use atoms_provider::Provider;
    pub use atoms_transport::Transport;
}
