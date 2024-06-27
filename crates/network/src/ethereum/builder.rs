use crate::{
    BuildResult, Ethereum, Network, NetworkSigner, TransactionBuilder, TransactionBuilderError,
};
use alloy_consensus::{BlobTransactionSidecar, TxType, TypedTransaction};
use alloy_primitives::{Bytes, ChainId, IcanAddress, TxKind, B1368, U256};
use alloy_rpc_types::{request::TransactionRequest, AccessList};
use alloy_signer::Signature;

impl TransactionBuilder<Ethereum> for TransactionRequest {
    fn network_id(&self) -> Option<ChainId> {
        self.network_id
    }

    fn set_network_id(&mut self, network_id: ChainId) {
        self.network_id = Some(network_id);
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
        self.input.input = Some(input.into());
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

    fn blob_sidecar(&self) -> Option<&BlobTransactionSidecar> {
        self.sidecar.as_ref()
    }

    fn set_blob_sidecar(&mut self, sidecar: BlobTransactionSidecar) {
        self.sidecar = Some(sidecar);
        self.populate_blob_hashes();
    }

    fn complete_type(&self, ty: TxType) -> Result<(), Vec<&'static str>> {
        match ty {
            TxType::Legacy => self.complete_legacy(),
            TxType::Eip2930 => self.complete_2930(),
            TxType::Eip1559 => self.complete_1559(),
            TxType::Eip4844 => self.complete_4844(),
        }
    }

    fn can_submit(&self) -> bool {
        // value and data may be None. If they are, they will be set to default.
        // gas fields and nonce may be None, if they are, they will be populated
        // with default values by the RPC server
        self.from.is_some()
    }

    fn can_build(&self) -> bool {
        // value and data may be none. If they are, they will be set to default
        // values.

        // chain_id and from may be none.
        let common = self.energy.is_some() && self.nonce.is_some();
        let legacy = self.energy_price.is_some();
        let eip2930 = legacy;

        let eip1559 = self.max_fee_per_gas.is_some() && self.max_priority_fee_per_gas.is_some();

        let eip4844 = eip1559 && self.sidecar.is_some() && self.to.is_some();
        common && (legacy || eip2930 || eip1559 || eip4844)
    }

    fn output_tx_type(&self) -> TxType {
        self.preferred_type()
    }

    fn output_tx_type_checked(&self) -> Option<TxType> {
        self.buildable_type()
    }

    fn prep_for_submission(&mut self) {
        self.transaction_type = Some(self.preferred_type() as u8);
        self.trim_conflicting_keys();
        self.populate_blob_hashes();
    }

    fn build_unsigned(self) -> BuildResult<TypedTransaction, Ethereum> {
        if let Err((tx_type, missing)) = self.missing_keys() {
            return Err((
                self,
                TransactionBuilderError::InvalidTransactionRequest(tx_type, missing),
            ));
        }
        Ok(self.build_typed_tx().expect("checked by missing_keys"))
    }

    async fn build<S: NetworkSigner<Ethereum>>(
        self,
        signer: &S,
    ) -> Result<<Ethereum as Network>::TxEnvelope, TransactionBuilderError<Ethereum>> {
        Ok(signer.sign_request(self).await?)
    }
}

#[cfg(test)]
mod tests {
    use crate::{TransactionBuilder, TransactionBuilderError};
    use alloy_consensus::{BlobTransactionSidecar, TxEip1559, TxType, TypedTransaction};
    use alloy_primitives::{Address, IcanAddress};
    use alloy_rpc_types::{AccessList, TransactionRequest};

    #[test]
    fn from_eip1559_to_tx_req() {
        let tx = TxEip1559 {
            chain_id: 1,
            nonce: 0,
            gas_limit: 21_000,
            to: IcanAddress::ZERO.into(),
            max_priority_fee_per_gas: 20e9 as u128,
            max_fee_per_gas: 20e9 as u128,
            ..Default::default()
        };
        let tx_req: TransactionRequest = tx.into();
        tx_req.build_unsigned().unwrap();
    }

    #[test]
    fn test_4844_when_sidecar() {
        let request = TransactionRequest::default()
            .with_nonce(1)
            .with_energy_limit(0)
            .with_max_fee_per_gas(0)
            .with_max_priority_fee_per_gas(0)
            .with_to(IcanAddress::ZERO)
            .with_blob_sidecar(BlobTransactionSidecar::default())
            .with_max_fee_per_blob_gas(0);

        let tx = request.clone().build_unsigned().unwrap();

        assert!(matches!(tx, TypedTransaction::Eip4844(_)));

        let tx = request.with_energy_price(0).build_unsigned().unwrap();

        assert!(matches!(tx, TypedTransaction::Eip4844(_)));
    }

    #[test]
    fn test_2930_when_access_list() {
        let request = TransactionRequest::default()
            .with_nonce(1)
            .with_energy_limit(0)
            .with_max_fee_per_gas(0)
            .with_max_priority_fee_per_gas(0)
            .with_to(IcanAddress::ZERO)
            .with_energy_price(0);
        // .with_access_list(AccessList::default());

        let tx = request.build_unsigned().unwrap();

        assert!(matches!(tx, TypedTransaction::Eip2930(_)));
    }

    #[test]
    fn test_default_to_1559() {
        let request = TransactionRequest::default()
            .with_nonce(1)
            .with_energy_limit(0)
            .with_max_fee_per_gas(0)
            .with_max_priority_fee_per_gas(0)
            .with_to(IcanAddress::ZERO);

        let tx = request.clone().build_unsigned().unwrap();

        assert!(matches!(tx, TypedTransaction::Eip1559(_)));

        let request = request.with_energy_price(0);
        let tx = request.build_unsigned().unwrap();
        assert!(matches!(tx, TypedTransaction::Legacy(_)));
    }

    #[test]
    fn test_fail_when_sidecar_and_access_list() {
        let request =
            TransactionRequest::default().with_blob_sidecar(BlobTransactionSidecar::default());
        // .with_access_list(AccessList::default());

        let error = request.clone().build_unsigned().unwrap_err();

        assert!(matches!(error.1, TransactionBuilderError::InvalidTransactionRequest(_, _)));
    }

    #[test]
    fn test_invalid_legacy_fields() {
        let request = TransactionRequest::default().with_energy_price(0);

        let error = request.clone().build_unsigned().unwrap_err();

        let (_, TransactionBuilderError::InvalidTransactionRequest(tx_type, errors)) = error else {
            panic!("wrong variant")
        };

        assert_eq!(tx_type, TxType::Legacy);
        assert_eq!(errors.len(), 3);
        assert!(errors.contains(&"to"));
        assert!(errors.contains(&"nonce"));
        assert!(errors.contains(&"gas_limit"));
    }

    #[test]
    fn test_invalid_1559_fields() {
        let request = TransactionRequest::default();

        let error = request.clone().build_unsigned().unwrap_err();

        let (_, TransactionBuilderError::InvalidTransactionRequest(tx_type, errors)) = error else {
            panic!("wrong variant")
        };

        assert_eq!(tx_type, TxType::Eip1559);
        assert_eq!(errors.len(), 5);
        assert!(errors.contains(&"to"));
        assert!(errors.contains(&"nonce"));
        assert!(errors.contains(&"gas_limit"));
        assert!(errors.contains(&"max_priority_fee_per_gas"));
        assert!(errors.contains(&"max_fee_per_gas"));
    }

    #[test]
    fn test_invalid_2930_fields() {
        let request = TransactionRequest::default()
            // .with_access_list(AccessList::default())
            .with_energy_price(Default::default());

        let error = request.clone().build_unsigned().unwrap_err();

        let (_, TransactionBuilderError::InvalidTransactionRequest(tx_type, errors)) = error else {
            panic!("wrong variant")
        };

        assert_eq!(tx_type, TxType::Eip2930);
        assert_eq!(errors.len(), 3);
        assert!(errors.contains(&"to"));
        assert!(errors.contains(&"nonce"));
        assert!(errors.contains(&"energy_limit"));
    }

    #[test]
    fn test_invalid_4844_fields() {
        let request =
            TransactionRequest::default().with_blob_sidecar(BlobTransactionSidecar::default());

        let error = request.clone().build_unsigned().unwrap_err();

        let (_, TransactionBuilderError::InvalidTransactionRequest(tx_type, errors)) = error else {
            panic!("wrong variant")
        };

        assert_eq!(tx_type, TxType::Eip4844);
        assert_eq!(errors.len(), 7);
        assert!(errors.contains(&"to"));
        assert!(errors.contains(&"nonce"));
        assert!(errors.contains(&"energy_limit"));
        assert!(errors.contains(&"max_priority_fee_per_gas"));
        assert!(errors.contains(&"max_fee_per_gas"));
        assert!(errors.contains(&"to"));
        assert!(errors.contains(&"max_fee_per_blob_gas"));
    }
}
