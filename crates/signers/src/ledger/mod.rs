pub mod app;
pub mod types;

use crate::Signer;
use app::LedgerEthereum;
use async_trait::async_trait;
use ethers_core::types::{
    transaction::{eip2718::TypedTransaction, eip712::Eip712},
    Address, Signature,
};
use types::LedgerError;

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Signer for LedgerEthereum {
    type Error = LedgerError;

    async fn sign_message(&self, message: &[u8]) -> Result<Signature, Self::Error> {
        self.sign_message(message).await
    }

    async fn sign_transaction(&self, message: &TypedTransaction) -> Result<Signature, Self::Error> {
        let mut tx_with_chain = message.clone();
        if tx_with_chain.chain_id().is_none() {
            // in the case we don't have a chain_id, let's use the signer chain id instead
            tx_with_chain.set_chain_id(self.chain_id);
        }
        self.sign_tx(&tx_with_chain).await
    }

    #[cfg(TODO)]
    async fn sign_typed_data<T: Eip712 + Send + Sync>(
        &self,
        payload: &T,
    ) -> Result<Signature, Self::Error> {
        self.sign_typed_struct(payload).await
    }

    fn address(&self) -> Address {
        self.address
    }

    fn with_chain_id<T: Into<u64>>(mut self, chain_id: T) -> Self {
        self.chain_id = chain_id.into();
        self
    }

    fn chain_id(&self) -> u64 {
        self.chain_id
    }
}
