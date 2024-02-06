use alloy_eips::eip2930::AccessList;
use alloy_network::{Builder, BuilderError, CanBuild, TxKind};
use alloy_primitives::{Bytes, ChainId, B256, U256};

use crate::{Ethereum, TxEip1559, TxEip2930, TxEip4844, TxLegacy};

/// A builder for Ethereum transactions.
#[derive(Default, Debug, Clone)]
pub struct EthereumTxBuilder {
    nonce: Option<u64>,
    gas_limit: Option<u64>,
    to: Option<TxKind>,
    value: Option<U256>,
    input: Option<Bytes>,
    chain_id: Option<ChainId>,

    // Legacy or 2930 Only
    gas_price: Option<u128>,

    // 1559 or 4844 Only
    max_priority_fee_per_gas: Option<u128>,
    max_fee_per_gas: Option<u128>,

    // 2930 or 4844 Only
    access_list: Option<AccessList>,

    // 4844 Only
    blob_versioned_hashes: Option<Vec<B256>>,
    max_fee_per_blob_gas: Option<u128>,
}

impl Builder<Ethereum> for EthereumTxBuilder {
    fn chain_id(&self) -> Option<alloy_primitives::ChainId> {
        self.chain_id
    }

    fn set_chain_id(&mut self, chain_id: alloy_primitives::ChainId) {
        self.chain_id = Some(chain_id);
    }

    fn nonce(&self) -> Option<u64> {
        self.nonce
    }

    fn set_nonce(&mut self, nonce: u64) {
        self.nonce = Some(nonce);
    }

    fn input(&self) -> Option<&Bytes> {
        self.input.as_ref()
    }

    fn set_input(&mut self, input: Bytes) {
        self.input = Some(input);
    }

    fn to(&self) -> Option<TxKind> {
        self.to
    }

    fn set_to(&mut self, to: TxKind) {
        self.to = Some(to);
    }

    fn value(&self) -> Option<U256> {
        self.value
    }

    fn set_value(&mut self, value: U256) {
        self.value = Some(value);
    }

    fn gas_price(&self) -> Option<u128> {
        self.gas_price
    }

    fn set_gas_price(&mut self, gas_price: u128) {
        self.gas_price = Some(gas_price);
    }

    fn gas_limit(&self) -> Option<u64> {
        self.gas_limit
    }

    fn set_gas_limit(&mut self, gas_limit: u64) {
        self.gas_limit = Some(gas_limit);
    }

    fn build_request(
        self,
    ) -> Result<<Ethereum as alloy_network::Network>::TransactionRequest, BuilderError> {
        todo!()
    }
}

impl EthereumTxBuilder {
    /// Create a new Ethereum transaction builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set max priority fee per gas for EIP-1559 or EIP-4844 transactions.
    pub fn set_max_priority_fee_per_gas(&mut self, max_priority_fee_per_gas: u128) {
        self.max_priority_fee_per_gas = Some(max_priority_fee_per_gas);
    }

    /// Builder-pattern method for setting max priority fee per gas.
    pub fn max_priority_fee_per_gas(mut self, max_priority_fee_per_gas: u128) -> Self {
        self.set_max_priority_fee_per_gas(max_priority_fee_per_gas);
        self
    }

    /// Set max fee per gas for EIP-1559 or EIP-4844 transactions.
    pub fn set_max_fee_per_gas(&mut self, max_fee_per_gas: u128) {
        self.max_fee_per_gas = Some(max_fee_per_gas);
    }

    /// Builder-pattern method for setting max fee per gas.
    pub fn max_fee_per_gas(mut self, max_fee_per_gas: u128) -> Self {
        self.set_max_fee_per_gas(max_fee_per_gas);
        self
    }

    /// Set access list for EIP-2930 or EIP-4844 transactions.
    pub fn set_access_list(&mut self, access_list: AccessList) {
        self.access_list = Some(access_list);
    }

    /// Builder-pattern method for setting access list.
    pub fn access_list(mut self, access_list: AccessList) -> Self {
        self.set_access_list(access_list);
        self
    }

    /// Set blob versioned hashes for EIP-4844 transactions.
    pub fn set_blob_versioned_hashes(&mut self, blob_versioned_hashes: Vec<B256>) {
        self.blob_versioned_hashes = Some(blob_versioned_hashes);
    }

    /// Builder-pattern method for setting blob versioned hashes.
    pub fn blob_versioned_hashes(mut self, blob_versioned_hashes: Vec<B256>) -> Self {
        self.set_blob_versioned_hashes(blob_versioned_hashes);
        self
    }

    /// Set max fee per blob gas for EIP-4844 transactions.
    pub fn set_max_fee_per_blob_gas(&mut self, max_fee_per_blob_gas: u128) {
        self.max_fee_per_blob_gas = Some(max_fee_per_blob_gas);
    }

    /// Builder-pattern method for setting max fee per blob gas.
    pub fn max_fee_per_blob_gas(mut self, max_fee_per_blob_gas: u128) -> Self {
        self.set_max_fee_per_blob_gas(max_fee_per_blob_gas);
        self
    }
}

impl CanBuild<TxLegacy> for EthereumTxBuilder {
    fn build(self) -> Result<TxLegacy, BuilderError> {
        Ok(TxLegacy {
            chain_id: self.chain_id,
            nonce: self.nonce.ok_or_else(|| BuilderError::MissingKey("nonce"))?,
            gas_price: self.gas_price.ok_or_else(|| BuilderError::MissingKey("gas_price"))?,
            gas_limit: self.gas_limit.ok_or_else(|| BuilderError::MissingKey("gas_limit"))?,
            to: self.to.ok_or_else(|| BuilderError::MissingKey("to"))?,
            value: self.value.unwrap_or_default(),
            input: self.input.unwrap_or_default(),
        })
    }
}

impl CanBuild<TxEip1559> for EthereumTxBuilder {
    fn build(self) -> Result<TxEip1559, BuilderError> {
        Ok(TxEip1559 {
            chain_id: self.chain_id.unwrap_or(1),
            nonce: self.nonce.ok_or_else(|| BuilderError::MissingKey("nonce"))?,
            max_priority_fee_per_gas: self
                .max_priority_fee_per_gas
                .ok_or_else(|| BuilderError::MissingKey("max_priority_fee_per_gas"))?,
            max_fee_per_gas: self
                .max_fee_per_gas
                .ok_or_else(|| BuilderError::MissingKey("max_fee_per_gas"))?,
            gas_limit: self.gas_limit.ok_or_else(|| BuilderError::MissingKey("gas_limit"))?,
            to: self.to.ok_or_else(|| BuilderError::MissingKey("to"))?,
            value: self.value.unwrap_or_default(),
            input: self.input.unwrap_or_default(),
            access_list: self.access_list.unwrap_or_default(),
        })
    }
}

impl CanBuild<TxEip2930> for EthereumTxBuilder {
    fn build(self) -> Result<TxEip2930, BuilderError> {
        Ok(TxEip2930 {
            chain_id: self.chain_id.unwrap_or(1),
            nonce: self.nonce.ok_or_else(|| BuilderError::MissingKey("nonce"))?,
            gas_price: self.gas_price.ok_or_else(|| BuilderError::MissingKey("gas_price"))?,
            gas_limit: self.gas_limit.ok_or_else(|| BuilderError::MissingKey("gas_limit"))?,
            to: self.to.ok_or_else(|| BuilderError::MissingKey("to"))?,
            value: self.value.unwrap_or_default(),
            input: self.input.unwrap_or_default(),
            access_list: self.access_list.unwrap_or_default(),
        })
    }
}

impl CanBuild<TxEip4844> for EthereumTxBuilder {
    fn build(self) -> Result<TxEip4844, BuilderError> {
        Ok(TxEip4844 {
            chain_id: self.chain_id.unwrap_or(1),
            nonce: self.nonce.ok_or_else(|| BuilderError::MissingKey("nonce"))?,
            gas_limit: self.gas_limit.ok_or_else(|| BuilderError::MissingKey("gas_limit"))?,
            max_fee_per_gas: self
                .max_fee_per_gas
                .ok_or_else(|| BuilderError::MissingKey("max_fee_per_gas"))?,
            max_priority_fee_per_gas: self
                .max_priority_fee_per_gas
                .ok_or_else(|| BuilderError::MissingKey("max_priority_fee_per_gas"))?,
            to: self.to.ok_or_else(|| BuilderError::MissingKey("to"))?,
            value: self.value.unwrap_or_default(),
            access_list: self.access_list.unwrap_or_default(),
            blob_versioned_hashes: self
                .blob_versioned_hashes
                .ok_or_else(|| BuilderError::MissingKey("blob_versioned_hashes"))?,
            max_fee_per_blob_gas: self
                .max_fee_per_blob_gas
                .ok_or_else(|| BuilderError::MissingKey("max_fee_per_blob_gas"))?,
            input: self.input.unwrap_or_default(),
        })
    }
}
