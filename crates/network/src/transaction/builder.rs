use super::signer::NetworkSigner;
use crate::Network;
use atoms_consensus::{Signed, TxLegacy, TypedTransaction};
use atoms_rpc_types::AccessList;
use atoms_signer::Signature;
use base_primitives::{Bytes, ChainId, IcanAddress, TxKind, B1368, U256};
use base_ylm_types::YlmCall;
use futures_utils_wasm::impl_future;

/// Result type for transaction builders
pub type BuildResult<T, N> = Result<T, Unbuilt<N>>;

/// An unbuilt transaction, along with some error.
pub type Unbuilt<N> = (<N as Network>::TransactionRequest, TransactionBuilderError);

/// Error type for transaction builders.
#[derive(Debug, thiserror::Error)]
pub enum TransactionBuilderError {
    /// Invalid transaction request
    #[error("Transaction can't be built due to missing keys: {0:?}")]
    InvalidTransactionRequest(Vec<&'static str>),

    /// Signer cannot produce signature type required for transaction.
    #[error("Signer cannot produce signature type required for transaction")]
    UnsupportedSignatureType,

    /// Signer error.
    #[error(transparent)]
    Signer(#[from] atoms_signer::Error),

    /// A custom error.
    #[error("{0}")]
    Custom(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
}

impl TransactionBuilderError {
    /// Instantiate a custom error.
    pub fn custom<E>(e: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Custom(Box::new(e))
    }
}

/// A Transaction builder for a network.
///
/// Transaction builders are primarily used to construct typed transactions that can be signed with
/// [`TransactionBuilder::build`], or unsigned typed transactions with
/// [`TransactionBuilder::build_unsigned`].
///
/// Transaction builders should be able to construct all available transaction types on a given
/// network.
pub trait TransactionBuilder<N: Network>: Default + Sized + Send + Sync + 'static {
    /// Get the network ID for the transaction.
    fn network_id(&self) -> ChainId;

    /// Set the network ID for the transaction.
    fn set_network_id(&mut self, chain_id: ChainId);

    /// Builder-pattern method for setting the network ID.
    fn with_network_id(mut self, network_id: base_primitives::ChainId) -> Self {
        self.set_network_id(network_id);
        self
    }

    /// Get the nonce for the transaction.
    fn nonce(&self) -> Option<u64>;

    /// Set the nonce for the transaction.
    fn set_nonce(&mut self, nonce: u64);

    /// Builder-pattern method for setting the nonce.
    fn with_nonce(mut self, nonce: u64) -> Self {
        self.set_nonce(nonce);
        self
    }

    /// Get the input data for the transaction.
    fn input(&self) -> Option<&Bytes>;

    /// Set the input data for the transaction.
    fn set_input<T: Into<Bytes>>(&mut self, input: T);

    /// Builder-pattern method for setting the input data.
    fn with_input<T: Into<Bytes>>(mut self, input: T) -> Self {
        self.set_input(input);
        self
    }

    /// Get the sender for the transaction.
    fn from(&self) -> Option<IcanAddress>;

    /// Set the sender for the transaction.
    fn set_from(&mut self, from: IcanAddress);

    /// Builder-pattern method for setting the sender.
    fn with_from(mut self, from: IcanAddress) -> Self {
        self.set_from(from);
        self
    }

    /// Get the kind of transaction.
    fn kind(&self) -> Option<base_primitives::TxKind>;

    /// Clear the kind of transaction.
    fn clear_kind(&mut self);

    /// Set the kind of transaction.
    fn set_kind(&mut self, kind: base_primitives::TxKind);

    /// Builder-pattern method for setting the kind of transaction.
    fn with_kind(mut self, kind: base_primitives::TxKind) -> Self {
        self.set_kind(kind);
        self
    }

    /// Get the recipient for the transaction.
    fn to(&self) -> Option<IcanAddress> {
        if let Some(TxKind::Call(addr)) = self.kind() {
            return Some(addr);
        }
        None
    }

    /// Set the recipient for the transaction.
    fn set_to(&mut self, to: IcanAddress) {
        self.set_kind(to.into());
    }

    /// Builder-pattern method for setting the recipient.
    fn with_to(mut self, to: IcanAddress) -> Self {
        self.set_to(to);
        self
    }

    /// Set the `to` field to a create call.
    fn set_create(&mut self) {
        self.set_kind(TxKind::Create);
    }

    /// Set the `to` field to a create call.
    fn into_create(mut self) -> Self {
        self.set_create();
        self
    }

    /// Deploy the code by making a create call with data. This will set the
    /// `to` field to [`TxKind::Create`].
    fn set_deploy_code<T: Into<Bytes>>(&mut self, code: T) {
        self.set_input(code.into());
        self.set_create()
    }

    /// Deploy the code by making a create call with data. This will set the
    /// `to` field to [`TxKind::Create`].
    fn with_deploy_code<T: Into<Bytes>>(mut self, code: T) -> Self {
        self.set_deploy_code(code);
        self
    }

    /// Set the data field to a contract call. This will clear the `to` field
    /// if it is set to [`TxKind::Create`].
    fn set_call<T: YlmCall>(&mut self, t: &T) {
        self.set_input(t.abi_encode());
        if matches!(self.kind(), Some(TxKind::Create)) {
            self.clear_kind();
        }
    }

    /// Make a contract call with data.
    fn with_call<T: YlmCall>(mut self, t: &T) -> Self {
        self.set_call(t);
        self
    }

    /// Calculates the address that will be created by the transaction, if any.
    ///
    /// Returns `None` if the transaction is not a contract creation (the `to` field is set), or if
    /// the `from` or `nonce` fields are not set.
    fn calculate_create_address(&self) -> Option<IcanAddress> {
        if !self.kind().is_some_and(|to| to.is_create()) {
            return None;
        }
        let from = self.from()?;
        let nonce = self.nonce()?;
        Some(from.create(nonce))
    }

    /// Get the value for the transaction.
    fn value(&self) -> Option<U256>;

    /// Set the value for the transaction.
    fn set_value(&mut self, value: U256);

    /// Builder-pattern method for setting the value.
    fn with_value(mut self, value: U256) -> Self {
        self.set_value(value);
        self
    }

    // /// Get the signature for the transaction.
    // fn signature(&self) -> Option<Signature>;

    // /// Set the signature for the transaction.
    // fn set_signature(&mut self, signature: Signature);

    // /// Builder-pattern method for setting the signature.
    // fn with_signature(mut self, signature: Signature) -> Self {
    //     self.set_signature(signature);
    //     self
    // }

    /// Get the legacy energy price for the transaction.
    fn energy_price(&self) -> Option<u128>;

    /// Set the legacy energy price for the transaction.
    fn set_energy_price(&mut self, energy_price: u128);

    /// Builder-pattern method for setting the legacy energy price.
    fn with_energy_price(mut self, energy_price: u128) -> Self {
        self.set_energy_price(energy_price);
        self
    }

    /// Get the max fee per gas for the transaction.
    fn max_fee_per_gas(&self) -> Option<u128>;

    /// Set the max fee per gas  for the transaction.
    fn set_max_fee_per_gas(&mut self, max_fee_per_gas: u128);

    /// Builder-pattern method for setting max fee per gas .
    fn with_max_fee_per_gas(mut self, max_fee_per_gas: u128) -> Self {
        self.set_max_fee_per_gas(max_fee_per_gas);
        self
    }

    /// Get the max priority fee per gas for the transaction.
    fn max_priority_fee_per_gas(&self) -> Option<u128>;

    /// Set the max priority fee per gas for the transaction.
    fn set_max_priority_fee_per_gas(&mut self, max_priority_fee_per_gas: u128);

    /// Builder-pattern method for setting max priority fee per gas.
    fn with_max_priority_fee_per_gas(mut self, max_priority_fee_per_gas: u128) -> Self {
        self.set_max_priority_fee_per_gas(max_priority_fee_per_gas);
        self
    }

    /// Get the max fee per blob gas for the transaction.
    fn max_fee_per_blob_gas(&self) -> Option<u128>;

    /// Set the max fee per blob gas  for the transaction.
    fn set_max_fee_per_blob_gas(&mut self, max_fee_per_blob_gas: u128);

    /// Builder-pattern method for setting max fee per blob gas .
    fn with_max_fee_per_blob_gas(mut self, max_fee_per_blob_gas: u128) -> Self {
        self.set_max_fee_per_blob_gas(max_fee_per_blob_gas);
        self
    }

    /// Get the energy limit for the transaction.
    fn energy_limit(&self) -> Option<u128>;

    /// Set the energy limit for the transaction.
    fn set_energy_limit(&mut self, energy_limit: u128);

    /// Builder-pattern method for setting the energy limit.
    fn with_energy_limit(mut self, energy_limit: u128) -> Self {
        self.set_energy_limit(energy_limit);
        self
    }

    // /// Get the EIP-2930 access list for the transaction.
    // fn access_list(&self) -> Option<&AccessList>;

    // /// Sets the EIP-2930 access list.
    // fn set_access_list(&mut self, access_list: AccessList);

    // /// Builder-pattern method for setting the access list.
    // fn with_access_list(mut self, access_list: AccessList) -> Self {
    //     self.set_access_list(access_list);
    //     self
    // }

    // /// Gets the EIP-4844 blob sidecar of the transaction.
    // fn blob_sidecar(&self) -> Option<&BlobTransactionSidecar>;

    // /// Sets the EIP-4844 blob sidecar of the transaction.
    // ///
    // /// Note: This will also set the versioned blob hashes accordingly:
    // /// [BlobTransactionSidecar::versioned_hashes]
    // fn set_blob_sidecar(&mut self, sidecar: BlobTransactionSidecar);

    // /// Builder-pattern method for setting the EIP-4844 blob sidecar of the transaction.
    // fn with_blob_sidecar(mut self, sidecar: BlobTransactionSidecar) -> Self {
    //     self.set_blob_sidecar(sidecar);
    //     self
    // }

    /// Check if all necessary keys are present to build the specified type,
    /// returning a list of missing keys.
    fn complete_type(&self) -> Result<(), Vec<&'static str>>;

    /// Check if all necessary keys are present to build the currently-preferred
    /// transaction type, returning a list of missing keys.
    fn complete_preferred(&self) -> Result<(), Vec<&'static str>> {
        self.complete_type()
    }

    /// True if the builder contains all necessary information to be submitted
    /// to the `eth_sendTransaction` endpoint.
    fn can_submit(&self) -> bool;

    /// True if the builder contains all necessary information to be built into
    /// a valid transaction.
    fn can_build(&self) -> bool;

    /// Trim any conflicting keys and populate any computed fields (like blob
    /// hashes).
    ///
    /// This is useful for transaction requests that have multiple conflicting
    /// fields. While these may be buildable, they may not be submitted to the
    /// RPC. This method should be called before RPC submission, but is not
    /// necessary before building.
    fn prep_for_submission(&mut self);

    /// Build an unsigned, but typed, transaction.
    fn build_unsigned(self) -> BuildResult<TypedTransaction, N>;

    /// Build a signed transaction.
    fn build<S: NetworkSigner<N>>(
        self,
        signer: &S,
    ) -> impl_future!(<Output = Result<Signed<TxLegacy, Signature>, TransactionBuilderError>>);
}
