use std::{future::IntoFuture, marker::PhantomData};

use alloy_dyn_abi::{DynSolValue, FunctionExt};
use alloy_json_abi::Function;
use alloy_network::Network;
use alloy_primitives::Bytes;
use alloy_rpc_types::{state::StateOverride, BlockId};
use alloy_sol_types::SolCall;
use alloy_transport::Transport;

use crate::{Error, Result};

mod private {
    pub trait Sealed {}
    impl Sealed for super::Function {}
    impl<C: super::SolCall> Sealed for super::PhantomData<C> {}
    impl Sealed for () {}
}

/// An [`alloy_provider::EthCall`] with an abi decoder.
#[derive(Debug, Clone)]
pub struct EthCall<'a, 'b, 'c, D, T, N>
where
    T: Transport + Clone,
    N: Network,
    D: CallDecoder,
{
    inner: alloy_provider::EthCall<'a, 'b, T, N>,

    decoder: &'c D,
}

impl<'a, 'b, 'c, D, T, N> EthCall<'a, 'b, 'c, D, T, N>
where
    T: Transport + Clone,
    N: Network,
    D: CallDecoder,
{
    /// Create a new [`EthCall`].
    pub fn new(inner: alloy_provider::EthCall<'a, 'b, T, N>, decoder: &'c D) -> Self {
        Self { inner, decoder }
    }

    /// Swap the decoder for this call.
    pub fn with_decoder<'e, E>(self, decoder: &'e E) -> EthCall<'a, 'b, 'e, E, T, N>
    where
        E: CallDecoder,
    {
        EthCall::new(self.inner, decoder)
    }

    /// Set the state overrides for this call.
    pub fn overrides(mut self, overrides: StateOverride) -> Self {
        self.inner = self.inner.overrides(overrides);
        self
    }

    /// Set the block to use for this call.
    pub fn block(mut self, block: BlockId) -> Self {
        self.inner = self.inner.block(block);
        self
    }
}

impl<'a, 'b, T, N> From<alloy_provider::EthCall<'a, 'b, T, N>>
    for EthCall<'a, 'b, 'static, (), T, N>
where
    T: Transport + Clone,
    N: Network,
{
    fn from(inner: alloy_provider::EthCall<'a, 'b, T, N>) -> Self {
        EthCall::new(inner, &())
    }
}

impl<'a, 'b, 'c, D, T, N> std::future::IntoFuture for EthCall<'a, 'b, 'c, D, T, N>
where
    D: CallDecoder + Unpin,
    T: Transport + Clone,
    N: Network,
{
    type Output = Result<D::CallOutput>;

    type IntoFuture = EthCallFut<'a, 'b, 'c, D, T, N>;

    fn into_future(self) -> Self::IntoFuture {
        EthCallFut { inner: self.inner.into_future(), decoder: self.decoder }
    }
}

/// Future for the [`EthCall`] type. This future wraps an RPC call with an abi
/// decoder.
#[derive(Debug, Clone)]
pub struct EthCallFut<'a, 'b, 'c, D, T, N>
where
    T: Transport + Clone,
    N: Network,
    D: CallDecoder,
{
    inner: <alloy_provider::EthCall<'a, 'b, T, N> as IntoFuture>::IntoFuture,
    decoder: &'c D,
}

impl<'a, 'b, 'c, D, T, N> std::future::Future for EthCallFut<'a, 'b, 'c, D, T, N>
where
    D: CallDecoder + Unpin,
    T: Transport + Clone,
    N: Network,
{
    type Output = Result<D::CallOutput>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.get_mut();
        let pin = std::pin::pin!(&mut this.inner);
        match pin.poll(cx) {
            std::task::Poll::Ready(Ok(data)) => {
                std::task::Poll::Ready(this.decoder.abi_decode_output(data, true))
            }
            std::task::Poll::Ready(Err(e)) => std::task::Poll::Ready(Err(e.into())),
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}

/// A trait for decoding the output of a contract function.
///
/// This trait is sealed and cannot be implemented manually.
/// It is an implementation detail of [`CallBuilder`].
///
/// [`CallBuilder`]: crate::CallBuilder
pub trait CallDecoder: private::Sealed {
    // Not public API.

    /// The output type of the contract function.
    #[doc(hidden)]
    type CallOutput;

    /// Decodes the output of a contract function.
    #[doc(hidden)]
    fn abi_decode_output(&self, data: Bytes, validate: bool) -> Result<Self::CallOutput>;

    #[doc(hidden)]
    fn as_debug_field(&self) -> impl std::fmt::Debug;
}

impl CallDecoder for Function {
    type CallOutput = Vec<DynSolValue>;

    #[inline]
    fn abi_decode_output(&self, data: Bytes, validate: bool) -> Result<Self::CallOutput> {
        FunctionExt::abi_decode_output(self, &data, validate).map_err(Error::AbiError)
    }

    #[inline]
    fn as_debug_field(&self) -> impl std::fmt::Debug {
        self
    }
}

impl<C: SolCall> CallDecoder for PhantomData<C> {
    type CallOutput = C::Return;

    #[inline]
    fn abi_decode_output(&self, data: Bytes, validate: bool) -> Result<Self::CallOutput> {
        C::abi_decode_returns(&data, validate).map_err(|e| Error::AbiError(e.into()))
    }

    #[inline]
    fn as_debug_field(&self) -> impl std::fmt::Debug {
        std::any::type_name::<C>()
    }
}

impl CallDecoder for () {
    type CallOutput = Bytes;

    #[inline]
    fn abi_decode_output(&self, data: Bytes, _validate: bool) -> Result<Self::CallOutput> {
        Ok(data)
    }

    #[inline]
    fn as_debug_field(&self) -> impl std::fmt::Debug {
        format_args!("()")
    }
}
