use crate::{Network, ReceiptResponse};

mod builder;

mod signer;
pub use signer::EthereumSigner;

/// Types for a mainnet-like Core network.
#[derive(Clone, Copy, Debug)]
pub struct Core {
    _private: (),
}

impl Network for Core {
    type TxType = alloy_consensus::TxType;

    type TxEnvelope = alloy_consensus::TxEnvelope;

    type UnsignedTx = alloy_consensus::TypedTransaction;

    type ReceiptEnvelope = alloy_consensus::ReceiptEnvelope;

    type Header = alloy_consensus::Header;

    type TransactionRequest = alloy_rpc_types::transaction::TransactionRequest;

    type TransactionResponse = alloy_rpc_types::Transaction;

    type ReceiptResponse = alloy_rpc_types::TransactionReceipt;

    type HeaderResponse = alloy_rpc_types::Header;
}

impl ReceiptResponse for alloy_rpc_types::TransactionReceipt {
    fn contract_address(&self) -> Option<alloy_primitives::IcanAddress> {
        self.contract_address
    }
}
