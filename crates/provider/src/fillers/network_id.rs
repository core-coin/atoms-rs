use std::sync::{Arc, OnceLock};

use alloy_network::{Network, TransactionBuilder};
use alloy_transport::TransportResult;

use crate::{
    fillers::{FillerControlFlow, TxFiller},
    provider::SendableTx,
};

/// A [`TxFiller`] that populates the network ID of a transaction.
///
/// If a network ID is provided, it will be used for filling. If a network ID
/// is not provided, the filler will attempt to fetch the network ID from the
/// provider the first time a transaction is prepared, and will cache it for
/// future transactions.
///
/// Transactions that already have a network_id set by the user will not be
/// modified.
///
/// # Example
///
/// ```
/// # use alloy_network::{NetworkSigner, EthereumSigner, Core};
/// # use alloy_rpc_types::TransactionRequest;
/// # use alloy_provider::{ProviderBuilder, RootProvider, Provider};
/// # async fn test<S: NetworkSigner<Ethereum> + Clone>(url: url::Url, signer: S) -> Result<(), Box<dyn std::error::Error>> {
/// let provider = ProviderBuilder::new()
///     .with_network_id(1)
///     .signer(signer)
///     .on_http(url);
///
/// provider.send_transaction(TransactionRequest::default()).await;
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NetworkIdFiller(Arc<OnceLock<u64>>);

impl NetworkIdFiller {
    /// Create a new [`NetworkIdFiller`] with an optional network ID.
    ///
    /// If a network ID is provided, it will be used for filling. If a network ID
    /// is not provided, the filler will attempt to fetch the network ID from the
    /// provider the first time a transaction is prepared.
    pub fn new(network_id: Option<u64>) -> Self {
        let lock = OnceLock::new();
        if let Some(network_id) = network_id {
            lock.set(network_id).expect("brand new");
        }
        Self(Arc::new(lock))
    }
}

impl<N: Network> TxFiller<N> for NetworkIdFiller {
    type Fillable = u64;

    fn status(&self, tx: &N::TransactionRequest) -> FillerControlFlow {
        if tx.network_id().is_some() {
            FillerControlFlow::Finished
        } else {
            FillerControlFlow::Ready
        }
    }

    async fn prepare<P, T>(
        &self,
        provider: &P,
        _tx: &N::TransactionRequest,
    ) -> TransportResult<Self::Fillable>
    where
        P: crate::Provider<T, N>,
        T: alloy_transport::Transport + Clone,
    {
        match self.0.get().copied() {
            Some(network_id) => Ok(network_id),
            None => {
                let network_id = provider.get_chain_id().await?;
                let network_id = *self.0.get_or_init(|| network_id);
                Ok(network_id)
            }
        }
    }

    async fn fill(
        &self,
        fillable: Self::Fillable,
        mut tx: SendableTx<N>,
    ) -> TransportResult<SendableTx<N>> {
        if let Some(builder) = tx.as_mut_builder() {
            if builder.network_id().is_none() {
                builder.set_network_id(fillable)
            }
        };
        Ok(tx)
    }
}
