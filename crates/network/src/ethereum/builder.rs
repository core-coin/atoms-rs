use crate::{BuildResult, Ethereum, NetworkSigner, TransactionBuilder, TransactionBuilderError};
use alloy_consensus::{Signed, TxLegacy, TypedTransaction};
use base_primitives::{Bytes, ChainId, IcanAddress, TxKind, U256};
use alloy_rpc_types::{request::TransactionRequest, TransactionInput};
use alloy_signer::Signature;

impl TransactionBuilder<Ethereum> for TransactionRequest {
    fn network_id(&self) -> ChainId {
        self.network_id
    }

    fn set_network_id(&mut self, network_id: ChainId) {
        self.network_id = network_id;
    }

    fn nonce(&self) -> Option<u64> {
        self.nonce
    }

    fn set_nonce(&mut self, nonce: u64) {
        self.nonce = Some(nonce);
    }

    fn input(&self) -> Option<&Bytes> {
        self.input.input()
    }

    fn set_input<T: Into<Bytes>>(&mut self, input: T) {
        self.input = TransactionInput { data: Some(input.into()), input: None };
    }

    fn from(&self) -> Option<IcanAddress> {
        self.from
    }

    fn set_from(&mut self, from: IcanAddress) {
        self.from = Some(from);
    }

    fn kind(&self) -> Option<TxKind> {
        self.to
    }

    fn set_kind(&mut self, kind: TxKind) {
        self.to = Some(kind);
    }

    fn clear_kind(&mut self) {
        self.to = None;
    }

    fn value(&self) -> Option<U256> {
        self.value
    }

    fn set_value(&mut self, value: U256) {
        self.value = Some(value)
    }

    // fn signature(&self) -> Option<Signature> {
    //     self.signature
    // }

    // fn set_signature(&mut self, signature: Signature) {
    //     self.signature = Some(signature)
    // }

    fn energy_price(&self) -> Option<u128> {
        self.energy_price
    }

    fn set_energy_price(&mut self, energy_price: u128) {
        self.energy_price = Some(energy_price);
    }

    fn max_fee_per_gas(&self) -> Option<u128> {
        self.max_fee_per_gas
    }

    fn set_max_fee_per_gas(&mut self, max_fee_per_gas: u128) {
        self.max_fee_per_gas = Some(max_fee_per_gas);
    }

    fn max_priority_fee_per_gas(&self) -> Option<u128> {
        self.max_priority_fee_per_gas
    }

    fn set_max_priority_fee_per_gas(&mut self, max_priority_fee_per_gas: u128) {
        self.max_priority_fee_per_gas = Some(max_priority_fee_per_gas);
    }

    fn max_fee_per_blob_gas(&self) -> Option<u128> {
        self.max_fee_per_blob_gas
    }

    fn set_max_fee_per_blob_gas(&mut self, max_fee_per_blob_gas: u128) {
        self.max_fee_per_blob_gas = Some(max_fee_per_blob_gas)
    }

    fn energy_limit(&self) -> Option<u128> {
        self.energy
    }

    fn set_energy_limit(&mut self, energy_limit: u128) {
        self.energy = Some(energy_limit);
    }

    // fn access_list(&self) -> Option<&AccessList> {
    //     self.access_list.as_ref()
    // }

    // fn set_access_list(&mut self, access_list: AccessList) {
    //     self.access_list = Some(access_list);
    // }

    // fn blob_sidecar(&self) -> Option<&BlobTransactionSidecar> {
    //     self.sidecar.as_ref()
    // }

    // fn set_blob_sidecar(&mut self, sidecar: BlobTransactionSidecar) {
    //     self.sidecar = Some(sidecar);
    //     self.populate_blob_hashes();
    // }

    fn complete_type(&self) -> Result<(), Vec<&'static str>> {
        self.complete_legacy()
    }

    fn can_submit(&self) -> bool {
        // value and data may be None. If they are, they will be set to default.
        // gas fields and nonce may be None, if they are, they will be populated
        // with default values by the RPC server
        self.from.is_some() && self.network_id != 0
    }

    fn can_build(&self) -> bool {
        // value and data may be none. If they are, they will be set to default
        // values.

        // from may be none.
        let common = self.energy.is_some() && self.nonce.is_some() && self.network_id != 0;
        let legacy = self.energy_price.is_some();
        let eip2930 = legacy;

        let eip1559 = self.max_fee_per_gas.is_some() && self.max_priority_fee_per_gas.is_some();

        let eip4844 = eip1559 && self.to.is_some();
        common && (legacy || eip2930 || eip1559 || eip4844)
    }

    fn prep_for_submission(&mut self) {}

    fn build_unsigned(self) -> BuildResult<TypedTransaction, Ethereum> {
        if let Err(missing) = self.complete_legacy() {
            return Err((self, TransactionBuilderError::InvalidTransactionRequest(missing)));
        }
        Ok(self.build_typed_tx().expect("checked by complete_legacy"))
    }

    async fn build<S: NetworkSigner<Ethereum>>(
        self,
        signer: &S,
    ) -> Result<Signed<TxLegacy, Signature>, TransactionBuilderError> {
        Ok(signer.sign_request(self).await?)
    }
}

#[cfg(test)]
mod tests {
    use crate::{TransactionBuilder, TransactionBuilderError};
    use alloy_consensus::{TxLegacy, TypedTransaction};
    use base_primitives::{Address, IcanAddress};
    use alloy_rpc_types::{AccessList, TransactionRequest};

    #[test]
    fn from_legacy_to_tx_req() {
        let tx = TxLegacy {
            network_id: 1,
            nonce: 0,
            energy_limit: 21_000,
            to: IcanAddress::ZERO.into(),
            energy_price: 1,
            ..Default::default()
        };
        let tx_req: TransactionRequest = tx.into();
        tx_req.build_unsigned().unwrap();
    }

    #[test]
    fn test_invalid_legacy_fields() {
        let request = TransactionRequest::default().with_energy_price(0);

        let error = request.clone().build_unsigned().unwrap_err();

        let (_, TransactionBuilderError::InvalidTransactionRequest(errors)) = error else {
            panic!("wrong variant")
        };

        assert_eq!(errors.len(), 4);
        assert!(errors.contains(&"to"));
        assert!(errors.contains(&"nonce"));
        assert!(errors.contains(&"energy_limit"));
        assert!(errors.contains(&"network_id"));
    }
}
