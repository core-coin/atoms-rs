//! Block hearbeat and pending transaction watcher.

use crate::Provider;
use alloy_network::Network;
use alloy_primitives::{B256, U256};
use alloy_rpc_types::Block;
use alloy_transport::{utils::Spawnable, Transport, TransportErrorKind, TransportResult};
use futures::{stream::StreamExt, FutureExt, Stream};
use std::{
    collections::{BTreeMap, HashMap},
    fmt,
    future::Future,
    marker::PhantomData,
    time::{Duration, Instant},
};
use tokio::{
    select,
    sync::{mpsc, oneshot, watch},
};

/// A configuration object for watching for transaction confirmation.
#[must_use = "this type does nothing unless you call `register`, `watch` or `get_receipt`"]
#[derive(Debug)]
pub struct PendingTransactionConfig<N, T, P> {
    inner: PendingTransactionConfigInner,
    provider: P,
    _phantom: PhantomData<(N, T)>,
}

impl<N: Network, T: Transport + Clone, P: Provider<N, T>> PendingTransactionConfig<N, T, P> {
    /// Creates a new pending transaction configuration.
    pub fn new(provider: P, tx_hash: B256) -> Self {
        Self::from_inner(provider, PendingTransactionConfigInner::new(tx_hash))
    }

    /// Creates a new pending transaction configuration.
    pub fn from_inner(provider: P, inner: PendingTransactionConfigInner) -> Self {
        Self { inner, provider, _phantom: PhantomData }
    }

    /// Returns the inner configuration.
    pub fn inner(&self) -> &PendingTransactionConfigInner {
        &self.inner
    }

    /// Consumes this configuration, returning the inner configuration.
    pub fn into_inner(self) -> PendingTransactionConfigInner {
        self.inner
    }

    /// Returns the provider.
    pub fn provider(&self) -> &P {
        &self.provider
    }

    /// Consumes this configuration, returning the provider and the inner configuration.
    pub fn split(self) -> (P, PendingTransactionConfigInner) {
        (self.provider, self.inner)
    }

    /// Returns the transaction hash.
    pub fn tx_hash(&self) -> &B256 {
        self.inner.tx_hash()
    }

    /// Sets the transaction hash.
    pub fn set_tx_hash(&mut self, tx_hash: B256) {
        self.inner.set_tx_hash(tx_hash);
    }

    /// Sets the transaction hash.
    pub fn with_tx_hash(mut self, tx_hash: B256) -> Self {
        self.set_tx_hash(tx_hash);
        self
    }

    /// Returns the number of confirmations to wait for.
    pub fn confirmations(&self) -> u64 {
        self.inner.confirmations()
    }

    /// Sets the number of confirmations to wait for.
    pub fn set_confirmations(&mut self, confirmations: u64) {
        self.inner.set_confirmations(confirmations);
    }

    /// Sets the number of confirmations to wait for.
    pub fn with_confirmations(mut self, confirmations: u64) -> Self {
        self.set_confirmations(confirmations);
        self
    }

    /// Returns the timeout.
    pub fn timeout(&self) -> Option<Duration> {
        self.inner.timeout()
    }

    /// Sets the timeout.
    pub fn set_timeout(&mut self, timeout: Option<Duration>) {
        self.inner.set_timeout(timeout);
    }

    /// Sets the timeout.
    pub fn with_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.set_timeout(timeout);
        self
    }

    /// Registers the watching configuration with the provider.
    ///
    /// This does not wait for the transaction to be confirmed, but returns a [`PendingTransaction`]
    /// that can be awaited at a later moment.
    ///
    /// See:
    /// - [`watch`](Self::watch) for watching the transaction without fetching the receipt.
    /// - [`get_receipt`](Self::get_receipt) for fetching the receipt after the transaction has been
    ///   confirmed.
    pub async fn register(self) -> TransportResult<PendingTransaction> {
        self.provider.watch_pending_transaction(self.inner).await
    }

    /// Waits for the transaction to confirm with the given number of confirmations.
    ///
    /// See:
    /// - [`register`](Self::register): for registering the transaction without waiting for it to be
    ///   confirmed.
    /// - [`get_receipt`](Self::get_receipt) for fetching the receipt after the transaction has been
    ///   confirmed.
    pub async fn watch(self) -> TransportResult<B256> {
        self.register().await?.await
    }

    /// Waits for the transaction to confirm with the given number of confirmations, and
    /// then fetches its receipt.
    ///
    /// See:
    /// - [`register`](Self::register): for registering the transaction without waiting for it to be
    ///   confirmed.
    /// - [`watch`](Self::watch) for watching the transaction without fetching the receipt.
    pub async fn get_receipt(self) -> TransportResult<Option<N::ReceiptResponse>> {
        let pending_tx = self.provider.watch_pending_transaction(self.inner).await?;
        let hash = pending_tx.await?;
        self.provider.get_transaction_receipt(hash).await
    }
}

impl<N, T, P: Clone> PendingTransactionConfig<N, T, &P> {
    /// Clones the provider and returns a new pending transaction configuration with the cloned
    /// provider.
    pub fn with_cloned_provider(self) -> PendingTransactionConfig<N, T, P> {
        PendingTransactionConfig {
            inner: self.inner,
            provider: self.provider.clone(),
            _phantom: PhantomData,
        }
    }
}

/// A configuration object for watching for transaction confirmation.
///
/// This object is not directly usable, but can be used to create a [`PendingTransactionConfig`]
/// to watch for a transaction.
#[must_use = "this type does nothing unless you call `with_provider`"]
#[derive(Debug)]
pub struct PendingTransactionConfigInner {
    /// The transaction hash to watch for.
    tx_hash: B256,

    /// Require a number of confirmations.
    confirmations: u64,

    /// Optional timeout for the transaction.
    timeout: Option<Duration>,
}

impl PendingTransactionConfigInner {
    /// Create a new watch for a transaction.
    pub fn new(tx_hash: B256) -> Self {
        Self { tx_hash, confirmations: 0, timeout: None }
    }

    /// Returns the transaction hash.
    pub fn tx_hash(&self) -> &B256 {
        &self.tx_hash
    }

    /// Sets the transaction hash.
    pub fn set_tx_hash(&mut self, tx_hash: B256) {
        self.tx_hash = tx_hash;
    }

    /// Sets the transaction hash.
    pub fn with_tx_hash(mut self, tx_hash: B256) -> Self {
        self.set_tx_hash(tx_hash);
        self
    }

    /// Returns the number of confirmations to wait for.
    pub fn confirmations(&self) -> u64 {
        self.confirmations
    }

    /// Sets the number of confirmations to wait for.
    pub fn set_confirmations(&mut self, confirmations: u64) {
        self.confirmations = confirmations;
    }

    /// Sets the number of confirmations to wait for.
    pub fn with_confirmations(mut self, confirmations: u64) -> Self {
        self.set_confirmations(confirmations);
        self
    }

    /// Returns the timeout.
    pub fn timeout(&self) -> Option<Duration> {
        self.timeout
    }

    /// Sets the timeout.
    pub fn set_timeout(&mut self, timeout: Option<Duration>) {
        self.timeout = timeout;
    }

    /// Sets the timeout.
    pub fn with_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.set_timeout(timeout);
        self
    }

    /// Wraps this configuration with a provider to expose watching methods.
    pub fn with_provider<N: Network, T: Transport + Clone, P: Provider<N, T>>(
        self,
        provider: P,
    ) -> PendingTransactionConfig<N, T, P> {
        PendingTransactionConfig::from_inner(provider, self)
    }
}

struct TxWatcher {
    config: PendingTransactionConfigInner,
    tx: oneshot::Sender<()>,
}

impl TxWatcher {
    /// Notify the waiter.
    fn notify(self) {
        debug!(tx=%self.config.tx_hash, "notifying");
        let _ = self.tx.send(());
    }
}

/// Represents a transaction that is either yet to be confirmed or has been confirmed
pub struct PendingTransaction {
    /// The transaction hash.
    pub(crate) tx_hash: B256,
    /// The receiver for the notification.
    // TODO: send a receipt?
    pub(crate) rx: oneshot::Receiver<()>,
}

impl fmt::Debug for PendingTransaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PendingTransaction").field("tx_hash", &self.tx_hash).finish()
    }
}

impl PendingTransaction {
    /// Returns this transaction's hash.
    pub const fn tx_hash(&self) -> &B256 {
        &self.tx_hash
    }
}

impl Future for PendingTransaction {
    type Output = TransportResult<B256>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        self.rx
            .poll_unpin(cx)
            .map(|res| res.map(|()| self.tx_hash).map_err(|_| TransportErrorKind::backend_gone()))
    }
}

/// A handle to the heartbeat task.
#[derive(Clone, Debug)]
pub(crate) struct HeartbeatHandle {
    tx: mpsc::Sender<TxWatcher>,
    #[allow(dead_code)]
    latest: watch::Receiver<Option<Block>>,
}

impl HeartbeatHandle {
    /// Watch for a transaction to be confirmed with the given config.
    pub(crate) async fn watch_tx(
        &self,
        config: PendingTransactionConfigInner,
    ) -> Result<PendingTransaction, PendingTransactionConfigInner> {
        let (tx, rx) = oneshot::channel();
        let tx_hash = config.tx_hash;
        match self.tx.send(TxWatcher { config, tx }).await {
            Ok(()) => Ok(PendingTransaction { tx_hash, rx }),
            Err(e) => Err(e.0.config),
        }
    }

    /// Returns a watcher that always sees the latest block.
    #[allow(dead_code)]
    pub(crate) fn latest(&self) -> &watch::Receiver<Option<Block>> {
        &self.latest
    }
}

// TODO: Parameterize with `Network`
/// A heartbeat task that receives blocks and watches for transactions.
pub(crate) struct Heartbeat<S> {
    /// The stream of incoming blocks to watch.
    stream: futures::stream::Fuse<S>,

    /// Transactions to watch for.
    unconfirmed: HashMap<B256, TxWatcher>,

    /// Ordered map of transactions waiting for confirmations.
    waiting_confs: BTreeMap<U256, Vec<TxWatcher>>,

    /// Ordered map of transactions to reap at a certain time.
    reap_at: BTreeMap<Instant, B256>,
}

impl<S: Stream<Item = Block>> Heartbeat<S> {
    /// Create a new heartbeat task.
    pub(crate) fn new(stream: S) -> Self {
        Self {
            stream: stream.fuse(),
            unconfirmed: Default::default(),
            waiting_confs: Default::default(),
            reap_at: Default::default(),
        }
    }
}

impl<S> Heartbeat<S> {
    /// Check if any transactions have enough confirmations to notify.
    fn check_confirmations(&mut self, current_height: &U256) {
        let to_keep = self.waiting_confs.split_off(current_height);
        let to_notify = std::mem::replace(&mut self.waiting_confs, to_keep);
        for watcher in to_notify.into_values().flatten() {
            watcher.notify();
        }
    }

    /// Get the next time to reap a transaction. If no reaps, this is a very
    /// long time from now (i.e. will not be woken).
    fn next_reap(&self) -> Instant {
        self.reap_at
            .first_key_value()
            .map(|(k, _)| *k)
            .unwrap_or_else(|| Instant::now() + Duration::from_secs(60_000))
    }

    /// Reap any timeout
    fn reap_timeouts(&mut self) {
        let now = Instant::now();
        let to_keep = self.reap_at.split_off(&now);
        let to_reap = std::mem::replace(&mut self.reap_at, to_keep);

        for tx_hash in to_reap.values() {
            if self.unconfirmed.remove(tx_hash).is_some() {
                debug!(tx=%tx_hash, "reaped");
            }
        }
    }

    /// Handle a watch instruction by adding it to the watch list, and
    /// potentially adding it to our `reap_at` list.
    fn handle_watch_ix(&mut self, to_watch: TxWatcher) {
        // Start watching for the transaction.
        debug!(tx=%to_watch.config.tx_hash, "watching");
        trace!(?to_watch.config);
        if let Some(timeout) = to_watch.config.timeout {
            self.reap_at.insert(Instant::now() + timeout, to_watch.config.tx_hash);
        }
        self.unconfirmed.insert(to_watch.config.tx_hash, to_watch);
    }

    /// Handle a new block by checking if any of the transactions we're
    /// watching are in it, and if so, notifying the watcher. Also updates
    /// the latest block.
    fn handle_new_block(&mut self, block: Block, latest: &watch::Sender<Option<Block>>) {
        // Blocks without numbers are ignored, as they're not part of the chain.
        let Some(block_height) = &block.header.number else { return };

        // Check if we are watching for any of the transactions in this block.
        let to_check =
            block.transactions.hashes().filter_map(|tx_hash| self.unconfirmed.remove(tx_hash));
        for watcher in to_check {
            // If `confirmations` is 0 we can notify the watcher immediately.
            let confirmations = watcher.config.confirmations;
            if confirmations == 0 {
                watcher.notify();
                continue;
            }
            // Otherwise add it to the waiting list.
            debug!(tx=%watcher.config.tx_hash, %block_height, confirmations, "adding to waiting list");
            self.waiting_confs
                .entry(*block_height + U256::from(confirmations))
                .or_default()
                .push(watcher);
        }

        self.check_confirmations(block_height);

        // Update the latest block. We use `send_replace` here to ensure the
        // latest block is always up to date, even if no receivers exist.
        // C.f. https://docs.rs/tokio/latest/tokio/sync/watch/struct.Sender.html#method.send
        debug!(%block_height, "updating latest block");
        let _ = latest.send_replace(Some(block));
    }
}

impl<S: Stream<Item = Block> + Unpin + Send + 'static> Heartbeat<S> {
    /// Spawn the heartbeat task, returning a [`HeartbeatHandle`]
    pub(crate) fn spawn(mut self) -> HeartbeatHandle {
        let (latest, latest_rx) = watch::channel(None::<Block>);
        let (ix_tx, mut ixns) = mpsc::channel(16);

        let fut = async move {
            'shutdown: loop {
                {
                    let next_reap = self.next_reap();
                    let sleep = std::pin::pin!(tokio::time::sleep_until(next_reap.into()));

                    // We bias the select so that we always handle new messages
                    // before checking blocks, and reap timeouts are last.
                    select! {
                        biased;

                        // Watch for new transactions.
                        ix_opt = ixns.recv() => match ix_opt {
                            Some(to_watch) => self.handle_watch_ix(to_watch),
                            None => break 'shutdown, // ix channel is closed
                        },

                        // Wake up to handle new blocks.
                        block = self.stream.select_next_some() => {
                            self.handle_new_block(block, &latest);
                        },

                        // This arm ensures we always wake up to reap timeouts,
                        // even if there are no other events.
                        _ = sleep => {},
                    }
                }

                // Always reap timeouts
                self.reap_timeouts();
            }
        };
        fut.spawn_task();

        HeartbeatHandle { tx: ix_tx, latest: latest_rx }
    }
}
