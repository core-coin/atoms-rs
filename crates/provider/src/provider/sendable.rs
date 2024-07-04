use alloy_consensus::{Signed, TxLegacy};
use alloy_network::Network;
use alloy_signer::Signature;

/// A transaction that can be sent. This is either a builder or an envelope.
///
/// This type is used to allow for fillers to convert a builder into an envelope
/// without changing the user-facing API.
///
/// Users should NOT use this type directly. It should only be used as an
/// implementation detail of [`Provider::send_transaction_internal`].
#[doc(hidden)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SendableTx<N: Network> {
    /// A transaction that is not yet signed.
    Builder(N::TransactionRequest),
    /// A transaction that is signed and fully constructed.
    Signed(Signed<TxLegacy, Signature>),
}

impl<N: Network> SendableTx<N> {
    /// Fallible cast to an unbuilt transaction request.
    pub fn as_mut_builder(&mut self) -> Option<&mut N::TransactionRequest> {
        match self {
            Self::Builder(tx) => Some(tx),
            _ => None,
        }
    }

    /// Fallible cast to an unbuilt transaction request.
    pub const fn as_builder(&self) -> Option<&N::TransactionRequest> {
        match self {
            Self::Builder(tx) => Some(tx),
            _ => None,
        }
    }

    /// Checks if the transaction is a builder.
    pub const fn is_builder(&self) -> bool {
        matches!(self, Self::Builder(_))
    }

    /// Check if the transaction is an envelope.
    pub const fn is_envelope(&self) -> bool {
        matches!(self, Self::Signed(_))
    }

    /// Fallible cast to a built transaction envelope.
    pub const fn as_envelope(&self) -> Option<&Signed<TxLegacy, Signature>> {
        match self {
            Self::Signed(tx) => Some(tx),
            _ => None,
        }
    }
}
