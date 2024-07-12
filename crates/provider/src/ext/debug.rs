//! This module extends the Core JSON-RPC provider with the Debug namespace's RPC methods.
use crate::Provider;
use atoms_network::Network;
use base_primitives::{TxHash, B256};
use atoms_rpc_types::{BlockNumberOrTag, TransactionRequest};
use atoms_rpc_types_trace::gocore::{
    GocoreDebugTracingCallOptions, GocoreDebugTracingOptions, GocoreTrace, TraceResult,
};
use atoms_transport::{Transport, TransportResult};

/// Debug namespace rpc interface that gives access to several non-standard RPC methods.
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
pub trait DebugApi<N, T>: Send + Sync {
    /// Reruns the transaction specified by the hash and returns the trace.
    ///
    /// It will replay any prior transactions to achieve the same state the transaction was executed
    /// in.
    ///
    /// [GocoreDebugTracingOptions] can be used to specify the trace options.
    ///
    /// # Note
    ///
    /// Not all nodes support this call.
    async fn debug_trace_transaction(
        &self,
        hash: TxHash,
        trace_options: GocoreDebugTracingOptions,
    ) -> TransportResult<GocoreTrace>;

    /// Return a full stack trace of all invoked opcodes of all transaction that were included in
    /// this block.
    ///
    /// The parent of the block must be present or it will fail.
    ///
    /// [GocoreDebugTracingOptions] can be used to specify the trace options.
    ///
    /// # Note
    ///
    /// Not all nodes support this call.
    async fn debug_trace_block_by_hash(
        &self,
        block: B256,
        trace_options: GocoreDebugTracingOptions,
    ) -> TransportResult<Vec<TraceResult>>;

    /// Same as `debug_trace_block_by_hash` but block is specified by number.
    ///
    /// [GocoreDebugTracingOptions] can be used to specify the trace options.
    ///
    /// # Note
    ///
    /// Not all nodes support this call.
    async fn debug_trace_block_by_number(
        &self,
        block: BlockNumberOrTag,
        trace_options: GocoreDebugTracingOptions,
    ) -> TransportResult<Vec<TraceResult>>;

    /// Executes the given transaction without publishing it like `eth_call` and returns the trace
    /// of the execution.
    ///
    /// The transaction will be executed in the context of the given block number or tag.
    /// The state its run on is the state of the previous block.
    ///
    /// [GocoreDebugTracingOptions] can be used to specify the trace options.
    ///
    /// # Note
    ///
    ///
    /// Not all nodes support this call.
    async fn debug_trace_call(
        &self,
        tx: TransactionRequest,
        block: BlockNumberOrTag,
        trace_options: GocoreDebugTracingCallOptions,
    ) -> TransportResult<GocoreTrace>;

    /// Same as `debug_trace_call` but it used to run and trace multiple transactions at once.
    ///
    /// [GocoreDebugTracingOptions] can be used to specify the trace options.
    ///
    /// # Note
    ///
    /// Not all nodes support this call.
    async fn debug_trace_call_many(
        &self,
        txs: Vec<TransactionRequest>,
        block: BlockNumberOrTag,
        trace_options: GocoreDebugTracingCallOptions,
    ) -> TransportResult<Vec<GocoreTrace>>;
}

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl<N, T, P> DebugApi<N, T> for P
where
    N: Network,
    T: Transport + Clone,
    P: Provider<T, N>,
{
    async fn debug_trace_transaction(
        &self,
        hash: TxHash,
        trace_options: GocoreDebugTracingOptions,
    ) -> TransportResult<GocoreTrace> {
        self.client().request("debug_traceTransaction", (hash, trace_options)).await
    }

    async fn debug_trace_block_by_hash(
        &self,
        block: B256,
        trace_options: GocoreDebugTracingOptions,
    ) -> TransportResult<Vec<TraceResult>> {
        self.client().request("debug_traceBlockByHash", (block, trace_options)).await
    }

    async fn debug_trace_block_by_number(
        &self,
        block: BlockNumberOrTag,
        trace_options: GocoreDebugTracingOptions,
    ) -> TransportResult<Vec<TraceResult>> {
        self.client().request("debug_traceBlockByNumber", (block, trace_options)).await
    }

    async fn debug_trace_call(
        &self,
        tx: TransactionRequest,
        block: BlockNumberOrTag,
        trace_options: GocoreDebugTracingCallOptions,
    ) -> TransportResult<GocoreTrace> {
        self.client().request("debug_traceCall", (tx, block, trace_options)).await
    }

    async fn debug_trace_call_many(
        &self,
        txs: Vec<TransactionRequest>,
        block: BlockNumberOrTag,
        trace_options: GocoreDebugTracingCallOptions,
    ) -> TransportResult<Vec<GocoreTrace>> {
        self.client().request("debug_traceCallMany", (txs, block, trace_options)).await
    }
}

#[cfg(test)]
mod test {
    use crate::{ProviderBuilder, WalletProvider};

    use super::*;
    use atoms_network::TransactionBuilder;
    use base_primitives::{cAddress, hex::FromHex, Bytes, U256};
    use atoms_rpc_types::TransactionInput;

    fn init_tracing() {
        let _ = tracing_subscriber::fmt::try_init();
    }

    #[tokio::test]
    async fn test_debug_trace_transaction() {
        init_tracing();
        let provider = ProviderBuilder::new().with_recommended_fillers().on_anvil_with_signer();
        let from = provider.default_signer_address();

        let energy_price = provider.get_energy_price().await.unwrap();
        let mut tx = TransactionRequest::default()
            .from(from)
            .to(cAddress!("cb7175017a3fa2d4dc29489bfc01ec2e60b140e1c019"))
            .value(U256::from(100));
        tx.set_energy_price(energy_price);
        tx.set_network_id(1);
        // .max_fee_per_gas(energy_price + 1)
        // .max_priority_fee_per_gas(energy_price + 1);
        let pending = provider.send_transaction(tx).await.unwrap();
        let receipt = pending.get_receipt().await.unwrap();

        let hash = receipt.transaction_hash;
        let trace_options = GocoreDebugTracingOptions::default();

        let trace = provider.debug_trace_transaction(hash, trace_options).await.unwrap();

        if let GocoreTrace::Default(trace) = trace {
            assert_eq!(trace.energy, 21000)
        }
    }

    #[tokio::test]
    async fn test_debug_trace_call() {
        init_tracing();
        let provider = ProviderBuilder::new().on_anvil_with_signer();
        let from = provider.default_signer_address();
        let energy_price = provider.get_energy_price().await.unwrap();
        let tx = TransactionRequest::default().from(from).input(TransactionInput {
            data: Some(Bytes::from_hex("0xdeadbeef").unwrap()),
            input: None,
        });
        // .max_fee_per_gas(energy_price + 1)
        // .max_priority_fee_per_gas(energy_price + 1);

        let trace = provider
            .debug_trace_call(
                tx,
                BlockNumberOrTag::Latest,
                GocoreDebugTracingCallOptions::default(),
            )
            .await
            .unwrap();

        if let GocoreTrace::Default(trace) = trace {
            assert!(!trace.struct_logs.is_empty());
        }
    }
}
