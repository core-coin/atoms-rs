//! RPC types for transactions

use std::str::FromStr;

use crate::eth::other::OtherFields;
use alloy_consensus::{SignableTransaction, Signed, TxLegacy};
use alloy_primitives::{Bytes, IcanAddress, Signature, TxKind, B256, U256};

use serde::{Deserialize, Serialize};

pub use alloy_eips::eip2930::{AccessList, AccessListItem, AccessListWithGasUsed};

mod common;
pub use common::TransactionInfo;

mod error;
pub use error::ConversionError;

pub mod optimism;
pub use optimism::OptimismTransactionReceiptFields;

mod receipt;
pub use alloy_consensus::{AnyReceiptEnvelope, Receipt, ReceiptWithBloom};
pub use receipt::{AnyTransactionReceipt, TransactionReceipt};

pub mod request;
pub use request::{TransactionInput, TransactionRequest};

/// Transaction object used in RPC
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "arbitrary"), derive(arbitrary::Arbitrary))]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    /// Hash
    pub hash: B256,
    /// Nonce
    #[serde(with = "alloy_serde::num::u64_via_ruint")]
    pub nonce: u64,
    /// Block hash
    #[serde(default)]
    pub block_hash: Option<B256>,
    /// Block number
    #[serde(default, with = "alloy_serde::num::u64_opt_via_ruint")]
    pub block_number: Option<u64>,
    /// Transaction Index
    #[serde(default, with = "alloy_serde::num::u64_opt_via_ruint")]
    pub transaction_index: Option<u64>,
    /// Sender
    pub from: IcanAddress,
    /// Recipient
    pub to: Option<IcanAddress>,
    /// Transferred value
    pub value: U256,
    /// Energy Price
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "alloy_serde::num::u128_opt_via_ruint"
    )]
    pub energy_price: Option<u128>,
    /// Energy amount
    #[serde(with = "alloy_serde::num::u128_via_ruint")]
    pub energy: u128,
    /// Max BaseFeePerGas the user is willing to pay.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "alloy_serde::num::u128_opt_via_ruint"
    )]
    pub max_fee_per_gas: Option<u128>,
    /// The miner's tip.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "alloy_serde::num::u128_opt_via_ruint"
    )]
    pub max_priority_fee_per_gas: Option<u128>,
    /// Configured max fee per blob gas for eip-4844 transactions
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "alloy_serde::num::u128_opt_via_ruint"
    )]
    pub max_fee_per_blob_gas: Option<u128>,
    /// Data
    pub input: Bytes,
    /// All _flattened_ fields of the transaction signature.
    ///
    /// Note: this is an option so special transaction types without a signature (e.g. <https://github.com/ethereum-optimism/optimism/blob/0bf643c4147b43cd6f25a759d331ef3a2a61a2a3/specs/deposits.md#the-deposited-transaction-type>) can be supported.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub signature: Option<Signature>,
    /// The network id of the transaction.
    #[serde(default, with = "alloy_serde::u64_opt_via_ruint")]
    pub network_id: Option<u64>,
    /// Contains the blob hashes for eip-4844 transactions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blob_versioned_hashes: Option<Vec<B256>>,
    /// EIP2930
    ///
    /// Pre-pay to warm storage access.
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub access_list: Option<AccessList>,
    /// EIP2718
    ///
    /// Transaction type,
    /// Some(3) for EIP-4844 transaction, Some(2) for EIP-1559 transaction,
    /// Some(1) for AccessList transaction, None or Some(0) for Legacy
    #[serde(
        default,
        rename = "type",
        skip_serializing_if = "Option::is_none",
        with = "alloy_serde::num::u8_opt_via_ruint"
    )]
    pub transaction_type: Option<u8>,

    /// Arbitrary extra fields.
    ///
    /// This captures fields that are not native to ethereum but included in ethereum adjacent networks, for example fields the [optimism `eth_getTransactionByHash` request](https://docs.alchemy.com/alchemy/apis/optimism/eth-gettransactionbyhash) returns additional fields that this type will capture
    #[serde(flatten)]
    pub other: OtherFields,
}

impl Transaction {
    /// Returns true if the transaction is a legacy or 2930 transaction.
    pub const fn is_legacy_energy(&self) -> bool {
        self.energy_price.is_none()
    }

    /// Converts [Transaction] into [TransactionRequest].
    ///
    /// During this conversion data for [TransactionRequest::sidecar] is not populated as it is not
    /// part of [Transaction].
    pub fn into_request(self) -> TransactionRequest {
        let energy_price = match (self.energy_price, self.max_fee_per_gas) {
            (Some(energy_price), None) => Some(energy_price),
            // EIP-1559 transactions include deprecated `gasPrice` field displaying gas used by
            // transaction.
            // Setting this field for resulted tx request will result in it being invalid
            (_, Some(_)) => None,
            // unreachable
            (None, None) => None,
        };

        let to = self.to.map(TxKind::Call);

        TransactionRequest {
            from: Some(self.from),
            to,
            energy: Some(self.energy),
            energy_price,
            value: Some(self.value),
            input: self.input.into(),
            nonce: Some(self.nonce),
            network_id: self.network_id,
            // access_list: self.access_list,
            transaction_type: self.transaction_type,
            max_fee_per_gas: self.max_fee_per_gas,
            max_priority_fee_per_gas: self.max_priority_fee_per_gas,
            max_fee_per_blob_gas: self.max_fee_per_blob_gas,
            // blob_versioned_hashes: self.blob_versioned_hashes,
            // sidecar: None,
        }
    }
}

impl TryFrom<Transaction> for Signed<TxLegacy> {
    type Error = ConversionError;

    fn try_from(tx: Transaction) -> Result<Self, Self::Error> {
        let signature = tx.signature.ok_or(ConversionError::MissingSignature)?;

        let tx = TxLegacy {
            network_id: tx.network_id,
            nonce: tx.nonce,
            energy_price: tx.energy_price.ok_or(ConversionError::MissingGasPrice)?,
            energy_limit: tx.energy,
            to: tx.to.into(),
            value: tx.value,
            input: tx.input,
        };
        Ok(tx.into_signed(signature))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use arbitrary::Arbitrary;
    use rand::Rng;

    #[test]
    fn arbitrary_transaction() {
        let mut bytes = [0u8; 1024];
        rand::thread_rng().fill(bytes.as_mut_slice());
        let _: Transaction =
            Transaction::arbitrary(&mut arbitrary::Unstructured::new(&bytes)).unwrap();
    }

    #[test]
    fn serde_transaction() {
        let transaction = Transaction {
            hash: B256::with_last_byte(1),
            nonce: 2,
            block_hash: Some(B256::with_last_byte(3)),
            block_number: Some(4),
            transaction_index: Some(5),
            from: IcanAddress::with_last_byte(6),
            to: Some(IcanAddress::with_last_byte(7)),
            value: U256::from(8),
            energy_price: Some(9),
            energy: 10,
            input: Bytes::from(vec![11, 12, 13]),
            signature: Some(Signature::from_str("0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000").unwrap()),
            network_id: Some(17),
            blob_versioned_hashes: None,
            // access_list: None,
            transaction_type: Some(20),
            max_fee_per_gas: Some(21),
            max_priority_fee_per_gas: Some(22),
            max_fee_per_blob_gas: None,
            other: Default::default(),
        };
        let serialized = serde_json::to_string(&transaction).unwrap();
        assert_eq!(
            serialized,
            r#"{"hash":"0x0000000000000000000000000000000000000000000000000000000000000001","nonce":"0x2","blockHash":"0x0000000000000000000000000000000000000000000000000000000000000003","blockNumber":"0x4","transactionIndex":"0x5","from":"0x00000000000000000000000000000000000000000006","to":"0x00000000000000000000000000000000000000000007","value":"0x8","energyPrice":"0x9","energy":"0xa","maxFeePerGas":"0x15","maxPriorityFeePerGas":"0x16","input":"0x0b0c0d","sig":"0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000","networkId":"0x11","type":"0x14"}"#
        );
        let deserialized: Transaction = serde_json::from_str(&serialized).unwrap();
        assert_eq!(transaction, deserialized);
    }

    #[test]
    fn serde_minimal_transaction() {
        let transaction = Transaction {
            hash: B256::with_last_byte(1),
            nonce: 2,
            from: IcanAddress::with_last_byte(6),
            value: U256::from(8),
            energy: 10,
            input: Bytes::from(vec![11, 12, 13]),
            ..Default::default()
        };
        let serialized = serde_json::to_string(&transaction).unwrap();
        assert_eq!(
            serialized,
            r#"{"hash":"0x0000000000000000000000000000000000000000000000000000000000000001","nonce":"0x2","blockHash":null,"blockNumber":null,"transactionIndex":null,"from":"0x00000000000000000000000000000000000000000006","to":null,"value":"0x8","energy":"0xa","input":"0x0b0c0d","networkId":null}"#
        );
        let deserialized: Transaction = serde_json::from_str(&serialized).unwrap();
        assert_eq!(transaction, deserialized);
    }

    #[test]
    fn into_request_legacy() {
        // cast rpc eth_getTransactionByHash
        // 0xe9e91f1ee4b56c0df2e9f06c2b8c27c6076195a88a7b8537ba8313d80e6f124e --rpc-url mainnet
        let rpc_tx = r#"{"blockHash":"0x8e38b4dbf6b11fcc3b9dee84fb7986e29ca0a02cecd8977c161ff7333329681e","blockNumber":"0xf4240","hash":"0xe9e91f1ee4b56c0df2e9f06c2b8c27c6076195a88a7b8537ba8313d80e6f124e","transactionIndex":"0x1","type":"0x0","nonce":"0x43eb","input":"0x","r":"0x3b08715b4403c792b8c7567edea634088bedcd7f60d9352b1f16c69830f3afd5","s":"0x10b9afb67d2ec8b956f0e1dbc07eb79152904f3a7bf789fc869db56320adfe09","networkId":"0x0","v":"0x1c","energy":"0xc350","from":"0x000032be343b94f860124dc4fee278fdcbd38c102d88","to":"0x0000df190dc7190dfba737d7777a163445b7fff16133","value":"0x6113a84987be800","energyPrice":"0xdf8475800"}"#;

        let tx = serde_json::from_str::<Transaction>(rpc_tx).unwrap();
        let request = tx.into_request();
        assert!(request.energy_price.is_some());
        assert!(request.max_fee_per_gas.is_none());
    }

    #[test]
    fn into_request_eip1559() {
        // cast rpc eth_getTransactionByHash
        // 0x0e07d8b53ed3d91314c80e53cf25bcde02084939395845cbb625b029d568135c --rpc-url mainnet
        let rpc_tx = r#"{"blockHash":"0x883f974b17ca7b28cb970798d1c80f4d4bb427473dc6d39b2a7fe24edc02902d","blockNumber":"0xe26e6d","hash":"0x0e07d8b53ed3d91314c80e53cf25bcde02084939395845cbb625b029d568135c","accessList":[],"transactionIndex":"0xad","type":"0x2","nonce":"0x16d","input":"0x5ae401dc00000000000000000000000000000000000000000000000000000000628ced5b000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000016000000000000000000000000000000000000000000000000000000000000000e442712a6700000000000000000000000000000000000000000000b3ff1489674e11c40000000000000000000000000000000000000000000000000000004a6ed55bbcc18000000000000000000000000000000000000000000000000000000000000000800000000000000000000000003cf412d970474804623bb4e3a42de13f9bca54360000000000000000000000000000000000000000000000000000000000000002000000000000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc20000000000000000000000003a75941763f31c930b19c041b709742b0b31ebb600000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000412210e8a00000000000000000000000000000000000000000000000000000000","r":"0x7f2153019a74025d83a73effdd91503ceecefac7e35dd933adc1901c875539aa","s":"0x334ab2f714796d13c825fddf12aad01438db3a8152b2fe3ef7827707c25ecab3","networkId":"0x1","v":"0x0","energy":"0x46a02","maxPriorityFeePerGas":"0x59682f00","from":"0x00003cf412d970474804623bb4e3a42de13f9bca5436","to":"0x000068b3465833fb72a70ecdf485e0e4c7bd8665fc45","maxFeePerGas":"0x7fc1a20a8","value":"0x4a6ed55bbcc180","energyPrice":"0x50101df3a"}"#;

        let tx = serde_json::from_str::<Transaction>(rpc_tx).unwrap();
        let request = tx.into_request();
        assert!(request.energy_price.is_none());
        assert!(request.max_fee_per_gas.is_some());
    }
}
