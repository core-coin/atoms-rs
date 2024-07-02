use crate::{Network, NetworkSigner, TxSigner};
use alloy_consensus::{SignableTransaction, Signed, TxLegacy, TypedTransaction};
use alloy_primitives::IcanAddress;
use alloy_signer::Signature;
use async_trait::async_trait;
use std::{collections::BTreeMap, sync::Arc};

/// A signer capable of signing any transaction for the Ethereum network.
#[derive(Clone, Default)]
pub struct EthereumSigner {
    default: IcanAddress,
    secp_signers: BTreeMap<IcanAddress, Arc<dyn TxSigner<Signature> + Send + Sync>>,
}

impl std::fmt::Debug for EthereumSigner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EthereumSigner")
            .field("default_signer", &self.default)
            .field("credentials", &self.secp_signers.len())
            .finish()
    }
}

impl<S> From<S> for EthereumSigner
where
    S: TxSigner<Signature> + Send + Sync + 'static,
{
    fn from(signer: S) -> Self {
        Self::new(signer)
    }
}

impl EthereumSigner {
    /// Create a new signer with the given signer as the default signer.
    pub fn new<S>(signer: S) -> Self
    where
        S: TxSigner<Signature> + Send + Sync + 'static,
    {
        let mut this = Self::default();
        this.register_default_signer(signer);
        this
    }

    /// Register a new signer on this object. This signer will be used to sign
    /// [`TransactionRequest`] and [`TypedTransaction`] object that specify the
    /// signer's address in the `from` field.
    ///
    /// [`TransactionRequest`]: alloy_rpc_types::TransactionRequest
    pub fn register_signer<S>(&mut self, signer: S)
    where
        S: TxSigner<Signature> + Send + Sync + 'static,
    {
        self.secp_signers.insert(signer.address(), Arc::new(signer));
    }

    /// Register a new signer on this object, and set it as the default signer.
    /// This signer will be used to sign [`TransactionRequest`] and
    /// [`TypedTransaction`] objects that do not specify a signer address in the
    /// `from` field.
    ///
    /// [`TransactionRequest`]: alloy_rpc_types::TransactionRequest
    pub fn register_default_signer<S>(&mut self, signer: S)
    where
        S: TxSigner<Signature> + Send + Sync + 'static,
    {
        self.default = signer.address();
        self.register_signer(signer);
    }

    /// Get the default signer.
    pub fn default_signer(&self) -> Arc<dyn TxSigner<Signature> + Send + Sync + 'static> {
        self.secp_signers.get(&self.default).cloned().expect("invalid signer")
    }

    /// Get the signer for the given address.
    pub fn signer_by_address(
        &self,
        address: IcanAddress,
    ) -> Option<Arc<dyn TxSigner<Signature> + Send + Sync + 'static>> {
        self.secp_signers.get(&address).cloned()
    }

    async fn sign_transaction_inner(
        &self,
        sender: IcanAddress,
        tx: &mut dyn SignableTransaction<Signature>,
    ) -> alloy_signer::Result<Signature> {
        self.signer_by_address(sender)
            .ok_or_else(|| {
                alloy_signer::Error::other(format!("Missing signing credential for {}", sender))
            })?
            .sign_transaction(tx)
            .await
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<N> NetworkSigner<N> for EthereumSigner
where
    N: Network,
{
    fn default_signer_address(&self) -> IcanAddress {
        self.default
    }

    fn has_signer_for(&self, address: &IcanAddress) -> bool {
        self.secp_signers.contains_key(address)
    }

    fn signer_addresses(&self) -> impl Iterator<Item = IcanAddress> {
        self.secp_signers.keys().copied()
    }

    async fn sign_transaction_from(
        &self,
        sender: IcanAddress,
        tx: TypedTransaction,
    ) -> alloy_signer::Result<Signed<TxLegacy, Signature>> {
        match tx {
            TypedTransaction::Legacy(mut t) => {
                let sig = self.sign_transaction_inner(sender, &mut t).await?;
                Ok(t.into_signed(sig).into())
            }
        }
    }
}
