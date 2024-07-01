use std::ops::{Deref, DerefMut};

use alloy_consensus::BlobTransactionSidecar;
use alloy_primitives::Bytes;
use alloy_rpc_types::{AccessList, TransactionRequest, WithOtherFields};

use crate::{any::AnyNetwork, BuildResult, Network, TransactionBuilder, TransactionBuilderError};

impl TransactionBuilder<AnyNetwork> for WithOtherFields<TransactionRequest> {
    fn network_id(&self) -> Option<alloy_primitives::ChainId> {
        self.deref().network_id()
    }

    fn set_network_id(&mut self, chain_id: alloy_primitives::ChainId) {
        self.deref_mut().set_network_id(chain_id)
    }

    fn nonce(&self) -> Option<u64> {
        self.deref().nonce()
    }

    fn set_nonce(&mut self, nonce: u64) {
        self.deref_mut().set_nonce(nonce)
    }

    fn input(&self) -> Option<&alloy_primitives::Bytes> {
        self.deref().input()
    }

    fn set_input<T: Into<Bytes>>(&mut self, input: T) {
        self.deref_mut().set_input(input);
    }

    fn from(&self) -> Option<alloy_primitives::IcanAddress> {
        self.deref().from()
    }

    fn set_from(&mut self, from: alloy_primitives::IcanAddress) {
        self.deref_mut().set_from(from);
    }

    fn kind(&self) -> Option<alloy_primitives::TxKind> {
        self.deref().kind()
    }

    fn clear_kind(&mut self) {
        self.deref_mut().clear_kind()
    }

    fn set_kind(&mut self, kind: alloy_primitives::TxKind) {
        self.deref_mut().set_kind(kind)
    }

    fn value(&self) -> Option<alloy_primitives::U256> {
        self.deref().value()
    }

    fn set_value(&mut self, value: alloy_primitives::U256) {
        self.deref_mut().set_value(value)
    }

    // fn signature(&self) -> Option<alloy_primitives::B1368> {
    //     self.deref().signature()
    // }

    // fn set_signature(&mut self, signature: alloy_primitives::B1368) {
    //     self.deref_mut().set_signature(signature)
    // }

    fn energy_limit(&self) -> Option<u128> {
        self.deref().energy_limit()
    }

    fn set_energy_limit(&mut self, energy_limit: u128) {
        self.deref_mut().set_energy_limit(energy_limit);
    }

    fn max_fee_per_gas(&self) -> Option<u128> {
        self.deref().max_fee_per_gas()
    }

    fn set_max_fee_per_gas(&mut self, max_fee_per_gas: u128) {
        self.deref_mut().set_max_fee_per_gas(max_fee_per_gas);
    }

    fn max_priority_fee_per_gas(&self) -> Option<u128> {
        self.deref().max_priority_fee_per_gas()
    }

    fn set_max_priority_fee_per_gas(&mut self, max_priority_fee_per_gas: u128) {
        self.deref_mut().set_max_priority_fee_per_gas(max_priority_fee_per_gas);
    }

    fn max_fee_per_blob_gas(&self) -> Option<u128> {
        self.deref().max_fee_per_blob_gas()
    }

    fn set_max_fee_per_blob_gas(&mut self, max_fee_per_blob_gas: u128) {
        self.deref_mut().set_max_fee_per_blob_gas(max_fee_per_blob_gas)
    }

    fn energy_price(&self) -> Option<u128> {
        self.deref().energy_price()
    }

    fn set_energy_price(&mut self, gas_price: u128) {
        self.deref_mut().set_energy_price(gas_price);
    }

    // /// Get the EIP-2930 access list for the transaction.
    // fn access_list(&self) -> Option<&AccessList> {
    //     self.deref().access_list()
    // }

    // /// Sets the EIP-2930 access list.
    // fn set_access_list(&mut self, access_list: AccessList) {
    //     self.deref_mut().set_access_list(access_list)
    // }

    fn blob_sidecar(&self) -> Option<&BlobTransactionSidecar> {
        self.deref().blob_sidecar()
    }

    fn set_blob_sidecar(&mut self, sidecar: BlobTransactionSidecar) {
        self.deref_mut().set_blob_sidecar(sidecar)
    }

    fn complete_type(&self, ty: <AnyNetwork as Network>::TxType) -> Result<(), Vec<&'static str>> {
        self.deref().complete_type(ty.try_into().map_err(|_| vec!["supported tx type"])?)
    }

    fn can_build(&self) -> bool {
        self.deref().can_build()
    }

    fn can_submit(&self) -> bool {
        self.deref().can_submit()
    }

    fn output_tx_type(&self) -> <AnyNetwork as Network>::TxType {
        self.deref().output_tx_type().into()
    }

    fn output_tx_type_checked(&self) -> Option<<AnyNetwork as Network>::TxType> {
        self.deref().output_tx_type_checked().map(Into::into)
    }

    fn prep_for_submission(&mut self) {
        self.deref_mut().prep_for_submission()
    }

    fn build_unsigned(self) -> BuildResult<<AnyNetwork as Network>::UnsignedTx, AnyNetwork> {
        if let Err(missing) = self.complete_legacy() {
            return Err((
                self,
                TransactionBuilderError::InvalidTransactionRequest(tx_type.into(), missing),
            ));
        }
        Ok(self.inner.build_typed_tx().expect("checked by complete_legacy"))
    }

    async fn build<S: crate::NetworkSigner<AnyNetwork>>(
        self,
        signer: &S,
    ) -> Result<<AnyNetwork as Network>::TxEnvelope, TransactionBuilderError<AnyNetwork>> {
        Ok(signer.sign_request(self).await?)
    }
}
