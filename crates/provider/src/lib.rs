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

#[cfg(any(test, feature = "reqwest"))]
/// Type alias for a [`RootProvider`] using the [`Http`] transport and a
/// reqwest client.
///
/// [`Http`]: atoms_transport_http::Http
pub type ReqwestProvider<N = atoms_network::Ethereum> =
    crate::RootProvider<atoms_transport_http::Http<reqwest::Client>, N>;

#[cfg(feature = "hyper")]
/// Type alias for a [`RootProvider`] using the [`Http`] transport and a hyper
/// client.
///
/// [`Http`]: atoms_transport_http::Http
pub type HyperProvider<N = atoms_network::Ethereum> =
    crate::RootProvider<atoms_transport_http::Http<atoms_transport_http::HyperClient>, N>;

#[macro_use]
extern crate tracing;

mod builder;
pub use builder::{Identity, ProviderBuilder, ProviderLayer, Stack};

pub mod ext;

pub mod fillers;
pub mod layers;

mod chain;

mod heart;
pub use heart::{PendingTransaction, PendingTransactionBuilder, PendingTransactionConfig};

mod provider;
pub use provider::{
    FilterPollerBuilder, Provider, RootProvider, SendableTx, WalletProvider, XcbCall,
};

pub mod utils;

#[doc(no_inline)]
pub use atoms_network::{self as network, Network};
