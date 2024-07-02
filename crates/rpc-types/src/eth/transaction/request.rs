//! Alloy basic Transaction Request type.

use crate::Transaction;
use alloy_consensus::{TxLegacy, TypedTransaction};
use alloy_primitives::{Address, Bytes, ChainId, IcanAddress, TxKind, B256, U256};
use serde::{Deserialize, Serialize};
use std::hash::Hash;

/// Represents _all_ transaction requests to/from RPC.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionRequest {
    /// The address of the transaction author.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<IcanAddress>,
    /// The destination address of the transaction.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<TxKind>,
    /// The legacy gas price.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "alloy_serde::num::u128_opt_via_ruint"
    )]
    pub energy_price: Option<u128>,
    /// The max base fee per gas the sender is willing to pay.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "alloy_serde::num::u128_opt_via_ruint"
    )]
    pub max_fee_per_gas: Option<u128>,
    /// The max priority fee per gas the sender is willing to pay, also called the miner tip.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "alloy_serde::num::u128_opt_via_ruint"
    )]
    pub max_priority_fee_per_gas: Option<u128>,
    /// The max fee per blob gas for EIP-4844 blob transactions.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "alloy_serde::num::u128_opt_via_ruint"
    )]
    pub max_fee_per_blob_gas: Option<u128>,
    /// The gas limit for the transaction.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "alloy_serde::num::u128_opt_via_ruint"
    )]
    pub energy: Option<u128>,
    /// The value transferred in the transaction, in wei.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<U256>,
    /// Transaction data.
    #[serde(default, flatten)]
    pub input: TransactionInput,
    /// The nonce of the transaction.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "alloy_serde::num::u64_opt_via_ruint"
    )]
    pub nonce: Option<u64>,
    /// The chain ID for the transaction.
    #[serde(default, with = "alloy_serde::num::u64_via_ruint")]
    pub network_id: ChainId,
    /// An EIP-2930 access list, which lowers cost for accessing accounts and storages in the list. See [EIP-2930](https://eips.ethereum.org/EIPS/eip-2930) for more information.
    // #[serde(default, skip_serializing_if = "Option::is_none")]
    // pub access_list: Option<AccessList>,
    /// The EIP-2718 transaction type. See [EIP-2718](https://eips.ethereum.org/EIPS/eip-2718) for more information.
    #[serde(
        default,
        rename = "type",
        skip_serializing_if = "Option::is_none",
        with = "alloy_serde::num::u8_opt_via_ruint"
    )]
    pub transaction_type: Option<u8>,
    // /// Blob versioned hashes for EIP-4844 transactions.
    // #[serde(default, skip_serializing_if = "Option::is_none")]
    // pub blob_versioned_hashes: Option<Vec<B256>>,
    // /// Blob sidecar for EIP-4844 transactions.
    // #[serde(default, skip_serializing_if = "Option::is_none")]
    // pub sidecar: Option<BlobTransactionSidecar>,
}

impl TransactionRequest {
    /// Sets the `from` field in the call to the provided address
    #[inline]
    pub const fn from(mut self, from: IcanAddress) -> Self {
        self.from = Some(from);
        self
    }

    /// Sets the transactions type for the transactions.
    pub const fn transaction_type(mut self, transaction_type: u8) -> Self {
        self.transaction_type = Some(transaction_type);
        self
    }

    /// Sets the energy limit for the transaction.
    pub const fn energy_limit(mut self, energy_limit: u128) -> Self {
        self.energy = Some(energy_limit);
        self
    }

    /// Sets the nonce for the transaction.
    pub const fn nonce(mut self, nonce: u64) -> Self {
        self.nonce = Some(nonce);
        self
    }

    /// Sets the maximum fee per gas for the transaction.
    pub const fn max_fee_per_gas(mut self, max_fee_per_gas: u128) -> Self {
        self.max_fee_per_gas = Some(max_fee_per_gas);
        self
    }

    /// Sets the maximum priority fee per gas for the transaction.
    pub const fn max_priority_fee_per_gas(mut self, max_priority_fee_per_gas: u128) -> Self {
        self.max_priority_fee_per_gas = Some(max_priority_fee_per_gas);
        self
    }

    /// Sets the recipient address for the transaction.
    #[inline]
    pub const fn to(mut self, to: IcanAddress) -> Self {
        self.to = Some(TxKind::Call(to));
        self
    }

    /// Sets the value (amount) for the transaction.
    pub const fn value(mut self, value: U256) -> Self {
        self.value = Some(value);
        self
    }

    // /// Sets the access list for the transaction.
    // pub fn access_list(mut self, access_list: AccessList) -> Self {
    //     self.access_list = Some(access_list);
    //     self
    // }

    /// Sets the input data for the transaction.
    pub fn input(mut self, input: TransactionInput) -> Self {
        self.input = input;
        self
    }

    /// Returns the configured fee cap, if any.
    ///
    /// The returns `gas_price` (legacy) if set or `max_fee_per_gas` (EIP1559)
    #[inline]
    pub fn fee_cap(&self) -> Option<u128> {
        self.energy_price.or(self.max_fee_per_gas)
    }

    /// Gets invalid fields for all transaction types
    pub fn get_invalid_common_fields(&self) -> Vec<&'static str> {
        let mut errors = vec![];

        if self.nonce.is_none() {
            errors.push("nonce");
        }

        if self.energy.is_none() {
            errors.push("energy_limit");
        }

        errors
    }

    /// Gets invalid fields for EIP-1559 transaction type
    pub fn get_invalid_1559_fields(&self) -> Vec<&'static str> {
        let mut errors = vec![];

        if self.max_priority_fee_per_gas.is_none() {
            errors.push("max_priority_fee_per_gas");
        }

        if self.max_fee_per_gas.is_none() {
            errors.push("max_fee_per_gas");
        }

        errors
    }

    /// Build a legacy transaction.
    ///
    /// # Panics
    ///
    /// If required fields are missing. Use `complete_legacy` to check if the
    /// request can be built.
    fn build_legacy(self) -> TxLegacy {
        let checked_to = self.to.expect("checked in complete_legacy.");

        TxLegacy {
            network_id: self.network_id,
            nonce: self.nonce.expect("checked in complete_legacy"),
            energy_price: self.energy_price.expect("checked in complete_legacy"),
            energy_limit: self.energy.expect("checked in complete_legacy"),
            to: checked_to,
            value: self.value.unwrap_or_default(),
            input: self.input.into_input().unwrap_or_default(),
        }
    }

    fn check_reqd_fields(&self) -> Vec<&'static str> {
        let mut missing = Vec::with_capacity(12);
        if self.nonce.is_none() {
            missing.push("nonce");
        }
        if self.energy.is_none() {
            missing.push("energy_limit");
        }
        if self.to.is_none() {
            missing.push("to");
        }
        if self.energy_price.is_none() {
            missing.push("energy_price");
        }
        missing
    }

    /// Check if all necessary keys are present to build a legacy transaction,
    /// returning a list of keys that are missing.
    pub fn complete_legacy(&self) -> Result<(), Vec<&'static str>> {
        let mut missing = self.check_reqd_fields();

        if missing.is_empty() {
            Ok(())
        } else {
            Err(missing)
        }
    }

    /// Build an [`TypedTransaction`]
    pub fn build_typed_tx(self) -> Result<TypedTransaction, Self> {
        let tx = self.build_legacy();
        Ok(TypedTransaction::Legacy(tx))
    }
}

/// Helper type that supports both `data` and `input` fields that map to transaction input data.
///
/// This is done for compatibility reasons where older implementations used `data` instead of the
/// newer, recommended `input` field.
///
/// If both fields are set, it is expected that they contain the same value, otherwise an error is
/// returned.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TransactionInput {
    /// Transaction data
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: Option<Bytes>,
    /// Transaction data
    ///
    /// This is the same as `input` but is used for backwards compatibility: <https://github.com/ethereum/go-ethereum/issues/15628>
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Bytes>,
}

impl TransactionInput {
    /// Creates a new instance with the given input data.
    pub const fn new(data: Bytes) -> Self {
        Self::maybe_input(Some(data))
    }

    /// Creates a new instance with the given input data.
    pub const fn maybe_input(input: Option<Bytes>) -> Self {
        Self { input, data: None }
    }

    /// Consumes the type and returns the optional input data.
    #[inline]
    pub fn into_input(self) -> Option<Bytes> {
        self.input.or(self.data)
    }

    /// Consumes the type and returns the optional input data.
    ///
    /// Returns an error if both `data` and `input` fields are set and not equal.
    #[inline]
    pub fn try_into_unique_input(self) -> Result<Option<Bytes>, TransactionInputError> {
        self.check_unique_input().map(|()| self.into_input())
    }

    /// Returns the optional input data.
    #[inline]
    pub fn input(&self) -> Option<&Bytes> {
        self.input.as_ref().or(self.data.as_ref())
    }

    /// Returns the optional input data.
    ///
    /// Returns an error if both `data` and `input` fields are set and not equal.
    #[inline]
    pub fn unique_input(&self) -> Result<Option<&Bytes>, TransactionInputError> {
        self.check_unique_input().map(|()| self.input())
    }

    fn check_unique_input(&self) -> Result<(), TransactionInputError> {
        if let (Some(input), Some(data)) = (&self.input, &self.data) {
            if input != data {
                return Err(TransactionInputError::default());
            }
        }
        Ok(())
    }
}

impl From<Vec<u8>> for TransactionInput {
    fn from(input: Vec<u8>) -> Self {
        Self { input: Some(input.into()), data: None }
    }
}

impl From<Bytes> for TransactionInput {
    fn from(input: Bytes) -> Self {
        Self { input: Some(input), data: None }
    }
}

impl From<Option<Bytes>> for TransactionInput {
    fn from(input: Option<Bytes>) -> Self {
        Self { input, data: None }
    }
}

impl From<Transaction> for TransactionRequest {
    fn from(tx: Transaction) -> Self {
        tx.into_request()
    }
}

impl From<TxLegacy> for TransactionRequest {
    fn from(tx: TxLegacy) -> Self {
        Self {
            to: if let TxKind::Call(to) = tx.to { Some(to.into()) } else { None },
            energy_price: Some(tx.energy_price),
            energy: Some(tx.energy_limit),
            value: Some(tx.value),
            input: tx.input.into(),
            nonce: Some(tx.nonce),
            network_id: tx.network_id,
            transaction_type: Some(0),
            ..Default::default()
        }
    }
}

impl From<TypedTransaction> for TransactionRequest {
    fn from(tx: TypedTransaction) -> Self {
        match tx {
            TypedTransaction::Legacy(tx) => tx.into(),
        }
    }
}

/// Error thrown when both `data` and `input` fields are set and not equal.
#[derive(Debug, Default, thiserror::Error)]
#[error("both \"data\" and \"input\" are set and not equal. Please use \"input\" to pass transaction call data")]
#[non_exhaustive]
pub struct TransactionInputError;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WithOtherFields;
    use alloy_primitives::b256;

    // <https://github.com/paradigmxyz/reth/issues/6670>
    #[test]
    fn serde_from_to() {
        let s = r#"{"from":"0x0000f39Fd6e51aad88F6F4ce6aB8827279cffFb92266", "to":"0x000070997970C51812dc3A010C7d01b50e0d17dc79C8" }"#;
        let req = serde_json::from_str::<TransactionRequest>(s).unwrap();
        assert!(req.input.check_unique_input().is_ok())
    }

    #[test]
    fn serde_tx_request() {
        let s = r#"{"accessList":[],"data":"0x0902f1ac","to":"0x0000a478c2975ab1ea89e8196811f51a7b7ade33eb11"}"#;
        let _req = serde_json::from_str::<TransactionRequest>(s).unwrap();
    }

    #[test]
    fn serde_unique_call_input() {
        let s = r#"{"accessList":[],"data":"0x0902f1ac", "input":"0x0902f1ac","to":"0x0000a478c2975ab1ea89e8196811f51a7b7ade33eb11","type":"0x02"}"#;
        let req = serde_json::from_str::<TransactionRequest>(s).unwrap();
        assert!(req.input.try_into_unique_input().unwrap().is_some());

        let s = r#"{"accessList":[],"data":"0x0902f1ac","to":"0x0000a478c2975ab1ea89e8196811f51a7b7ade33eb11","type":"0x02"}"#;
        let req = serde_json::from_str::<TransactionRequest>(s).unwrap();
        assert!(req.input.try_into_unique_input().unwrap().is_some());

        let s = r#"{"accessList":[],"input":"0x0902f1ac","to":"0x0000a478c2975ab1ea89e8196811f51a7b7ade33eb11","type":"0x02"}"#;
        let req = serde_json::from_str::<TransactionRequest>(s).unwrap();
        assert!(req.input.try_into_unique_input().unwrap().is_some());

        let s = r#"{"accessList":[],"data":"0x0902f1ac", "input":"0x0902f1","to":"0x0000a478c2975ab1ea89e8196811f51a7b7ade33eb11","type":"0x02"}"#;
        let req = serde_json::from_str::<TransactionRequest>(s).unwrap();
        assert!(req.input.try_into_unique_input().is_err());
    }

    #[test]
    fn serde_tx_request_additional_fields() {
        let s = r#"{"accessList":[],"data":"0x0902f1ac","to":"0x0000a478c2975ab1ea89e8196811f51a7b7ade33eb11","type":"0x02","sourceHash":"0xbf7e331f7f7c1dd2e05159666b3bf8bc7a8a3a9eb1d518969eab529dd9b88c1a"}"#;
        let req = serde_json::from_str::<WithOtherFields<TransactionRequest>>(s).unwrap();
        assert_eq!(
            req.other.get_deserialized::<B256>("sourceHash").unwrap().unwrap(),
            b256!("bf7e331f7f7c1dd2e05159666b3bf8bc7a8a3a9eb1d518969eab529dd9b88c1a")
        );
    }

    #[test]
    fn serde_tx_network_id_field() {
        let network_id: u64 = 12345678;

        let network_id_as_num = format!(r#"{{"networkId": {} }}"#, network_id);
        let req1 = serde_json::from_str::<TransactionRequest>(&network_id_as_num).unwrap();
        assert_eq!(req1.network_id, network_id);

        let network_id_as_hex = format!(r#"{{"networkId": "0x{:x}" }}"#, network_id);
        let req2 = serde_json::from_str::<TransactionRequest>(&network_id_as_hex).unwrap();
        assert_eq!(req2.network_id, network_id);
    }

    #[test]
    fn serde_empty() {
        let tx = TransactionRequest::default();
        let serialized = serde_json::to_string(&tx).unwrap();
        assert_eq!(serialized, "{\"networkId\":\"0x0\"}");
    }
}
