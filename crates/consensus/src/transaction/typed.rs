use crate::{transaction::TxLegacy, Transaction};
use base_primitives::TxKind;

/// The TypedTransaction enum represents all Ethereum transaction request types.
///
/// Its variants correspond to specific allowed transactions:
/// 1. Legacy (pre-EIP2718) [`TxLegacy`]
/// 2. EIP2930 (state access lists) [`TxEip2930`]
/// 3. EIP1559 [`TxEip1559`]
/// 4. EIP4844 [`TxEip4844Variant`]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
pub enum TypedTransaction {
    /// Legacy transaction
    #[cfg_attr(feature = "serde", serde(rename = "0x00", alias = "0x0"))]
    Legacy(TxLegacy),
}

impl From<TxLegacy> for TypedTransaction {
    fn from(tx: TxLegacy) -> Self {
        Self::Legacy(tx)
    }
}

impl TypedTransaction {
    /// Return the inner legacy transaction if it exists.
    pub const fn legacy(&self) -> Option<&TxLegacy> {
        match self {
            Self::Legacy(tx) => Some(tx),
        }
    }
}

impl Transaction for TypedTransaction {
    fn chain_id(&self) -> base_primitives::ChainId {
        match self {
            Self::Legacy(tx) => tx.chain_id(),
        }
    }

    fn gas_limit(&self) -> u128 {
        match self {
            Self::Legacy(tx) => tx.gas_limit(),
        }
    }

    fn gas_price(&self) -> Option<u128> {
        match self {
            Self::Legacy(tx) => tx.gas_price(),
        }
    }

    fn input(&self) -> &[u8] {
        match self {
            Self::Legacy(tx) => tx.input(),
        }
    }

    fn nonce(&self) -> u64 {
        match self {
            Self::Legacy(tx) => tx.nonce(),
        }
    }

    fn to(&self) -> TxKind {
        match self {
            Self::Legacy(tx) => tx.to(),
        }
    }

    fn value(&self) -> base_primitives::U256 {
        match self {
            Self::Legacy(tx) => tx.value(),
        }
    }
}
