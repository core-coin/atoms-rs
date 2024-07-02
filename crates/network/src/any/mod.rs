use core::fmt;

use crate::{Network, ReceiptResponse};
use alloy_consensus::TxLegacy;
use alloy_eips::eip2718::Eip2718Error;
use alloy_rpc_types::{
    AnyTransactionReceipt, Header, Transaction, TransactionRequest, WithOtherFields,
};

mod builder;

/// Essentially just returns the regular Ethereum types + a catch all field.
/// This [`Network`] should be used only when the network is not known at
/// compile time.
#[derive(Clone, Copy, Debug)]
pub struct AnyNetwork {
    _private: (),
}

impl Network for AnyNetwork {
    type ReceiptEnvelope = alloy_consensus::AnyReceiptEnvelope;

    type Header = alloy_consensus::Header;

    type TransactionRequest = WithOtherFields<TransactionRequest>;

    type TransactionResponse = WithOtherFields<Transaction>;

    type ReceiptResponse = AnyTransactionReceipt;

    type HeaderResponse = WithOtherFields<Header>;
}

impl ReceiptResponse for AnyTransactionReceipt {
    fn contract_address(&self) -> Option<alloy_primitives::IcanAddress> {
        self.contract_address
    }
}
