use crate::{Network, ReceiptResponse};

mod builder;

mod signer;
pub use signer::CoreSigner;

/// Types for a mainnet-like Core network.
#[derive(Clone, Copy, Debug)]
pub struct Ethereum {
    _private: (),
}

impl Network for Ethereum {
    type ReceiptEnvelope = atoms_consensus::AnyReceiptEnvelope;

    type Header = atoms_consensus::Header;

    type TransactionRequest = atoms_rpc_types::transaction::TransactionRequest;

    type TransactionResponse = atoms_rpc_types::Transaction;

    type ReceiptResponse = atoms_rpc_types::TransactionReceipt;

    type HeaderResponse = atoms_rpc_types::Header;
}

impl ReceiptResponse for atoms_rpc_types::TransactionReceipt {
    fn contract_address(&self) -> Option<base_primitives::IcanAddress> {
        self.contract_address
    }
}
