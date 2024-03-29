use crate::{PendingTransactionBuilder, Provider, ProviderLayer, RootProvider};
use alloy_network::{Ethereum, Network};
use alloy_transport::{Transport, TransportResult};
use async_trait::async_trait;
use futures::try_join;
use std::{future::Future, marker::PhantomData};

/// A layer that can fill in a `TransactionRequest` with additional information.
pub trait TxFiller<N: Network = Ethereum>: Clone + Send + Sync {
    /// The properties that this filler retrieves from the RPC. to fill in the
    /// TransactionRequest.
    type Fillable: Send + Sync + 'static;

    /// Joins this filler with another filler to compose multiple fillers.
    fn join_with<T>(self, other: T) -> JoinFill<Self, T, N>
    where
        T: TxFiller<N>,
    {
        JoinFill::new(self, other)
    }

    /// Returns true if the filler is ready to fill in the transaction request.
    fn ready(&self, tx: &N::TransactionRequest) -> bool;

    /// Returns true if all fillable properties have been filled.
    fn finished(&self, tx: &N::TransactionRequest) -> bool;

    /// Requests the fillable properties from the RPC.
    fn request<P, T>(
        &self,
        provider: &P,
        tx: &N::TransactionRequest,
    ) -> impl Future<Output = TransportResult<Self::Fillable>> + Send
    where
        P: Provider<T, N>,
        T: Transport + Clone;

    /// Fills in the transaction request with the fillable properties.
    fn fill(&self, fillable: Self::Fillable, tx: &mut N::TransactionRequest);
}

/// A layer that can fill in a `TransactionRequest` with additional information
/// by joining two [`FillTxLayer`]s. This  struct is itself a [`FillTxLayer`],
/// and can be nested to compose any number of fill layers.
#[derive(Debug, Clone)]
pub struct JoinFill<L, R, N> {
    left: L,
    right: R,
    _network: PhantomData<fn() -> N>,
}

impl<L, R, N> JoinFill<L, R, N>
where
    L: TxFiller<N>,
    R: TxFiller<N>,
    N: Network,
{
    /// Creates a new `JoinFill` with the given layers.
    pub fn new(left: L, right: R) -> Self {
        Self { left, right, _network: PhantomData }
    }

    /// Get a request for the left layer, if the left layer is ready.
    fn left_req<'a: 'd, 'b: 'd, 'c: 'd, 'd, P, T>(
        &'a self,
        provider: &'b P,
        tx: &'c N::TransactionRequest,
    ) -> impl Future<Output = TransportResult<Option<L::Fillable>>> + Send + 'd
    where
        P: Provider<T, N>,
        T: Transport + Clone,
    {
        async {
            if self.left.ready(tx) {
                self.left.request(provider, tx).await.map(Some)
            } else {
                Ok(None)
            }
        }
    }

    fn right_req<'a: 'd, 'b: 'd, 'c: 'd, 'd, P, T>(
        &'a self,
        provider: &'b P,
        tx: &'c N::TransactionRequest,
    ) -> impl Future<Output = TransportResult<Option<R::Fillable>>> + Send + 'd
    where
        P: Provider<T, N>,
        T: Transport + Clone,
    {
        async {
            if self.right.ready(tx) {
                self.right.request(provider, tx).await.map(Some)
            } else {
                Ok(None)
            }
        }
    }
}

impl<L, R, N> TxFiller<N> for JoinFill<L, R, N>
where
    L: TxFiller<N>,
    R: TxFiller<N>,
    N: Network,
{
    type Fillable = (Option<L::Fillable>, Option<R::Fillable>);

    fn ready(&self, tx: &N::TransactionRequest) -> bool {
        self.left.ready(tx) || self.right.ready(tx)
    }

    fn finished(&self, tx: &N::TransactionRequest) -> bool {
        self.left.finished(tx) && self.right.finished(tx)
    }

    fn request<P, T>(
        &self,
        provider: &P,
        tx: &N::TransactionRequest,
    ) -> impl Future<Output = TransportResult<Self::Fillable>> + Send
    where
        P: Provider<T, N>,
        T: Transport + Clone,
    {
        async { try_join!(self.left_req(provider, tx), self.right_req(provider, tx)) }
    }

    fn fill(&self, to_fill: Self::Fillable, tx: &mut N::TransactionRequest) {
        if let Some(to_fill) = to_fill.0 {
            self.left.fill(to_fill, tx);
        };
        if let Some(to_fill) = to_fill.1 {
            self.right.fill(to_fill, tx);
        };
    }
}

impl<L, R, P, T, N> ProviderLayer<P, T, N> for JoinFill<L, R, N>
where
    L: TxFiller<N>,
    R: TxFiller<N>,
    P: Provider<T, N>,
    T: alloy_transport::Transport + Clone,
    N: Network,
{
    type Provider = FillProvider<JoinFill<L, R, N>, P, T, N>;
    fn layer(&self, inner: P) -> Self::Provider {
        FillProvider::new(inner, self.clone())
    }
}

/// A [`Provider`] that joins or more [`FillTxLayer`]s.
///
/// Fills arbitrary properties in a transaction request by composing multiple
/// fill layers.

#[derive(Debug, Clone)]
pub struct FillProvider<F, P, T, N>
where
    F: TxFiller<N>,
    P: Provider<T, N>,
    T: Transport + Clone,
    N: Network,
{
    inner: P,
    filler: F,
    _pd: PhantomData<fn() -> (N, T)>,
}

impl<F, P, T, N> FillProvider<F, P, T, N>
where
    F: TxFiller<N>,
    P: Provider<T, N>,
    T: Transport + Clone,
    N: Network,
{
    /// Creates a new `FillProvider` with the given filler and inner provider.
    pub fn new(inner: P, filler: F) -> Self {
        Self { inner, filler, _pd: PhantomData }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<F, P, T, N> Provider<T, N> for FillProvider<F, P, T, N>
where
    F: TxFiller<N>,
    P: Provider<T, N>,
    T: Transport + Clone,
    N: Network,
{
    fn root(&self) -> &RootProvider<T, N> {
        self.inner.root()
    }

    async fn send_transaction(
        &self,
        mut tx: N::TransactionRequest,
    ) -> TransportResult<PendingTransactionBuilder<'_, T, N>> {
        while self.filler.ready(&tx) && !self.filler.finished(&tx) {
            let fillable = self.filler.request(self.root(), &tx).await?;
            self.filler.fill(fillable, &mut tx);
        }

        self.inner.send_transaction(tx).await
    }
}