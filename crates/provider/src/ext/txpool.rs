//! This modules extends the Core JSON-RPC provider with the Txpool namespace available in gocore.
use crate::Provider;
use atoms_network::{Ethereum, Network};
use atoms_rpc_types::txpool::{TxpoolContent, TxpoolContentFrom, TxpoolInspect, TxpoolStatus};
use atoms_transport::{Transport, TransportResult};
use base_primitives::IcanAddress;

/// Gocore only Txpool namespace rpc interface.
#[allow(unused, unreachable_pub)]
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
pub trait TxPoolApi<T, N = Ethereum>: Send + Sync {
    /// Returns the content of the transaction pool.
    ///
    /// Lists the exact details of all the transactions currently pending for inclusion in the next
    /// block(s), as well as the ones that are being scheduled for future execution only.
    ///
    /// See [here](https://gocore.ethereum.org/docs/rpc/ns-txpool#txpool_content) for more details
    async fn txpool_content(&self) -> TransportResult<TxpoolContent>;

    /// Returns the content of the transaction pool filtered by a specific address.
    ///
    /// See [here](https://gocore.ethereum.org/docs/rpc/ns-txpool#txpool_contentFrom) for more details
    // async fn txpool_content_from(&self, from: IcanAddress) -> TransportResult<TxpoolContentFrom>;

    /// Returns a textual summary of each transaction in the pool.
    ///
    /// Lists a textual summary of all the transactions currently pending for inclusion in the next
    /// block(s), as well as the ones that are being scheduled for future execution only.
    /// This is a method specifically tailored to developers to quickly see the
    /// transactions in the pool and find any potential issues.
    ///
    /// See [here](https://gocore.ethereum.org/docs/rpc/ns-txpool#txpool_inspect) for more details
    async fn txpool_inspect(&self) -> TransportResult<TxpoolInspect>;

    /// Returns the current status of the transaction pool.
    ///
    /// i.e the number of transactions currently pending for inclusion in the next block(s), as well
    /// as the ones that are being scheduled for future execution only.
    ///
    /// See [here](https://gocore.ethereum.org/docs/rpc/ns-txpool#txpool_status) for more details
    async fn txpool_status(&self) -> TransportResult<TxpoolStatus>;
}

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl<P, T, N> TxPoolApi<T, N> for P
where
    P: Provider<T, N>,
    T: Transport + Clone,
    N: Network,
{
    async fn txpool_content(&self) -> TransportResult<TxpoolContent> {
        self.client().request("txpool_content", ()).await
    }

    // async fn txpool_content_from(&self, from: IcanAddress) -> TransportResult<TxpoolContentFrom> {
    // self.client().request("txpool_contentFrom", (from,)).await
    // }

    async fn txpool_inspect(&self) -> TransportResult<TxpoolInspect> {
        self.client().request("txpool_inspect", ()).await
    }

    async fn txpool_status(&self) -> TransportResult<TxpoolStatus> {
        self.client().request("txpool_status", ()).await
    }
}

#[cfg(test)]
mod tests {
    use crate::ProviderBuilder;

    use super::*;
    use atoms_node_bindings::Gocore;

    #[tokio::test]
    async fn test_txpool_content() {
        let temp_dir = tempfile::TempDir::with_prefix("gocore-test-").unwrap();
        let gocore = Gocore::new().disable_discovery().data_dir(temp_dir.path()).spawn();
        let provider = ProviderBuilder::new().on_http(gocore.endpoint_url());
        let content = provider.txpool_content().await.unwrap();
        assert_eq!(content, TxpoolContent::default());
    }

    // #[tokio::test]
    // async fn test_txpool_content_from() {
    //     let temp_dir = tempfile::TempDir::with_prefix("gocore-test-").unwrap();
    //     let gocore = Gocore::new().disable_discovery().data_dir(temp_dir.path()).spawn();
    //     let provider = ProviderBuilder::new().on_http(gocore.endpoint_url());
    //     let content = provider.txpool_content_from(IcanAddress::default()).await.unwrap();
    //     assert_eq!(content, TxpoolContentFrom::default());
    // }

    #[tokio::test]
    async fn test_txpool_inspect() {
        let temp_dir = tempfile::TempDir::with_prefix("gocore-test-").unwrap();
        let gocore = Gocore::new().disable_discovery().data_dir(temp_dir.path()).spawn();
        let provider = ProviderBuilder::new().on_http(gocore.endpoint_url());
        let content = provider.txpool_inspect().await.unwrap();
        assert_eq!(content, TxpoolInspect::default());
    }

    #[tokio::test]
    async fn test_txpool_status() {
        let temp_dir = tempfile::TempDir::with_prefix("gocore-test-").unwrap();
        let gocore = Gocore::new().disable_discovery().data_dir(temp_dir.path()).spawn();
        let provider = ProviderBuilder::new().on_http(gocore.endpoint_url());
        let content = provider.txpool_status().await.unwrap();
        assert_eq!(content, TxpoolStatus::default());
    }
}
