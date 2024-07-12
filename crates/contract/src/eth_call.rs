use std::{future::IntoFuture, marker::PhantomData};

use atoms_network::Network;
use atoms_rpc_types::{state::StateOverride, BlockId};
use atoms_transport::Transport;
use base_dyn_abi::{DynYlmValue, FunctionExt};
use base_json_abi::Function;
use base_primitives::Bytes;
use base_ylm_types::YlmCall;

use crate::{Error, Result};

/// Raw coder.
const RAW_CODER: () = ();

mod private {
    pub trait Sealed {}
    impl Sealed for super::Function {}
    impl<C: super::YlmCall> Sealed for super::PhantomData<C> {}
    impl Sealed for () {}
}

/// An [`atoms_provider::EthCall`] with an abi decoder.
#[must_use = "EthCall must be awaited to execute the call"]
#[derive(Debug, Clone)]
pub struct XcbCall<'req, 'state, 'coder, D, T, N>
where
    T: Transport + Clone,
    N: Network,
    D: CallDecoder,
{
    inner: atoms_provider::XcbCall<'req, 'state, T, N>,

    decoder: &'coder D,
}

impl<'req, 'state, 'coder, D, T, N> XcbCall<'req, 'state, 'coder, D, T, N>
where
    T: Transport + Clone,
    N: Network,
    D: CallDecoder,
{
    /// Create a new [`EthCall`].
    pub const fn new(
        inner: atoms_provider::XcbCall<'req, 'state, T, N>,
        decoder: &'coder D,
    ) -> Self {
        Self { inner, decoder }
    }
}

impl<'req, 'state, T, N> XcbCall<'req, 'state, 'static, (), T, N>
where
    T: Transport + Clone,
    N: Network,
{
    /// Create a new [`EthCall`].
    pub const fn new_raw(inner: atoms_provider::XcbCall<'req, 'state, T, N>) -> Self {
        Self::new(inner, &RAW_CODER)
    }
}

impl<'req, 'state, 'coder, D, T, N> XcbCall<'req, 'state, 'coder, D, T, N>
where
    T: Transport + Clone,
    N: Network,
    D: CallDecoder,
{
    /// Swap the decoder for this call.
    #[allow(clippy::missing_const_for_fn)] // false positive
    pub fn with_decoder<'new_coder, E>(
        self,
        decoder: &'new_coder E,
    ) -> XcbCall<'req, 'state, 'new_coder, E, T, N>
    where
        E: CallDecoder,
    {
        XcbCall { inner: self.inner, decoder }
    }

    /// Set the state overrides for this call.
    pub fn overrides(mut self, overrides: &'state StateOverride) -> Self {
        self.inner = self.inner.overrides(overrides);
        self
    }

    /// Set the block to use for this call.
    pub fn block(mut self, block: BlockId) -> Self {
        self.inner = self.inner.block(block);
        self
    }
}

impl<'req, 'state, T, N> From<atoms_provider::XcbCall<'req, 'state, T, N>>
    for XcbCall<'req, 'state, 'static, (), T, N>
where
    T: Transport + Clone,
    N: Network,
{
    fn from(inner: atoms_provider::XcbCall<'req, 'state, T, N>) -> Self {
        Self { inner, decoder: &RAW_CODER }
    }
}

impl<'req, 'state, 'coder, D, T, N> std::future::IntoFuture
    for XcbCall<'req, 'state, 'coder, D, T, N>
where
    D: CallDecoder + Unpin,
    T: Transport + Clone,
    N: Network,
{
    type Output = Result<D::CallOutput>;

    type IntoFuture = EthCallFut<'req, 'state, 'coder, D, T, N>;

    fn into_future(self) -> Self::IntoFuture {
        EthCallFut { inner: self.inner.into_future(), decoder: self.decoder }
    }
}

/// Future for the [`EthCall`] type. This future wraps an RPC call with an abi
/// decoder.
#[must_use = "futures do nothing unless you `.await` or poll them"]
#[derive(Debug, Clone)]
pub struct EthCallFut<'req, 'state, 'coder, D, T, N>
where
    T: Transport + Clone,
    N: Network,
    D: CallDecoder,
{
    inner: <atoms_provider::XcbCall<'req, 'state, T, N> as IntoFuture>::IntoFuture,
    decoder: &'coder D,
}

impl<'req, 'state, 'coder, D, T, N> std::future::Future
    for EthCallFut<'req, 'state, 'coder, D, T, N>
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
    type CallOutput = Vec<DynYlmValue>;

    #[inline]
    fn abi_decode_output(&self, data: Bytes, validate: bool) -> Result<Self::CallOutput> {
        FunctionExt::abi_decode_output(self, &data, validate).map_err(Error::AbiError)
    }

    #[inline]
    fn as_debug_field(&self) -> impl std::fmt::Debug {
        self
    }
}

impl<C: YlmCall> CallDecoder for PhantomData<C> {
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
