use crate::{CallDecoder, Error, Result, XcbCall};
use atoms_network::{Ethereum, Network, ReceiptResponse, TransactionBuilder};
use atoms_provider::{PendingTransactionBuilder, Provider};
use atoms_rpc_types::{state::StateOverride, AccessList, BlockId};
use atoms_transport::Transport;
use base_dyn_abi::{DynYlmValue, JsonAbiExt};
use base_json_abi::Function;
use base_primitives::{Bytes, ChainId, IcanAddress, TxKind, U256};
use base_ylm_types::YlmCall;
use std::{
    future::{Future, IntoFuture},
    marker::PhantomData,
    pin::Pin,
};

/// [`CallBuilder`] using a [`YlmCall`] type as the call decoder.
// NOTE: please avoid changing this type due to its use in the `ylm!` macro.
pub type YlmCallBuilder<T, P, C, N = Ethereum> = CallBuilder<T, P, PhantomData<C>, N>;

/// [`CallBuilder`] using a [`Function`] as the call decoder.
pub type DynCallBuilder<T, P, N = Ethereum> = CallBuilder<T, P, Function, N>;

/// [`CallBuilder`] that does not have a call decoder.
pub type RawCallBuilder<T, P, N = Ethereum> = CallBuilder<T, P, (), N>;

/// A builder for sending a transaction via `eth_sendTransaction`, or calling a contract via
/// `eth_call`.
///
/// The builder can be `.await`ed directly, which is equivalent to invoking [`call`].
/// Prefer using [`call`] when possible, as `await`ing the builder directly will consume it, and
/// currently also boxes the future due to type system limitations.
///
/// A call builder can currently be instantiated in the following ways:
/// - by [`ylm!`][ylm]-generated contract structs' methods (through the `#[ylm(rpc)]` attribute)
///   ([`YlmCallBuilder`]);
/// - by [`ContractInstance`](crate::ContractInstance)'s methods ([`DynCallBuilder`]);
/// - using [`CallBuilder::new_raw`] ([`RawCallBuilder`]).
///
/// Each method represents a different way to decode the output of the contract call.
///
/// [`call`]: CallBuilder::call
///
/// # Note
///
/// This will set [state overrides](https://geth.ethereum.org/docs/rpc/ns-eth#3-object---state-override-set)
/// for `eth_call`, but this is not supported by all clients.
///
/// # Examples
///
/// Using [`ylm!`][ylm]:
///
/// ```no_run
/// # async fn test<P: base_contract::private::Provider>(provider: P) -> Result<(), Box<dyn std::error::Error>> {
/// use base_contract::YlmCallBuilder;
/// use base_primitives::{IcanAddress, U256};
/// use base_ylm_types::ylm;
///
/// ylm! {
///     #[ylm(rpc)] // <-- Important!
///     contract MyContract {
///         function doStuff(uint a, bool b) public returns(address c, bytes32 d);
///     }
/// }
///
/// # stringify!(
/// let provider = ...;
/// # );
/// let address = IcanAddress::ZERO;
/// let contract = MyContract::new(address, &provider);
///
/// // Through `contract.<function_name>(args...)`
/// let a = U256::ZERO;
/// let b = true;
/// let builder: YlmCallBuilder<_, _, MyContract::doStuffCall, _> = contract.doStuff(a, b);
/// let MyContract::doStuffReturn { c: _, d: _ } = builder.call().await?;
///
/// // Through `contract.call_builder(&<FunctionCall { args... }>)`:
/// // (note that this is discouraged because it's inherently less type-safe)
/// let call = MyContract::doStuffCall { a, b };
/// let builder: YlmCallBuilder<_, _, MyContract::doStuffCall, _> = contract.call_builder(&call);
/// let MyContract::doStuffReturn { c: _, d: _ } = builder.call().await?;
/// # Ok(())
/// # }
/// ```
///
/// Using [`ContractInstance`](crate::ContractInstance):
///
/// ```no_run
/// # async fn test<P: base_contract::private::Provider>(provider: P, dynamic_abi: base_json_abi::JsonAbi) -> Result<(), Box<dyn std::error::Error>> {
/// use base_primitives::{IcanAddress, Bytes, U256};
/// use base_dyn_abi::DynYlmValue;
/// use base_contract::{CallBuilder, ContractInstance, DynCallBuilder, Interface, RawCallBuilder};
///
/// # stringify!(
/// let dynamic_abi: JsonAbi = ...;
/// # );
/// let interface = Interface::new(dynamic_abi);
///
/// # stringify!(
/// let provider = ...;
/// # );
/// let address = IcanAddress::ZERO;
/// let contract: ContractInstance<_, _, _> = interface.connect(address, &provider);
///
/// // Build and call the function:
/// let call_builder: DynCallBuilder<_, _, _> = contract.function("doStuff", &[U256::ZERO.into(), true.into()])?;
/// let result: Vec<DynYlmValue> = call_builder.call().await?;
///
/// // You can also decode the output manually. Get the raw bytes:
/// let raw_result: Bytes = call_builder.call_raw().await?;
/// // Or, equivalently:
/// let raw_builder: RawCallBuilder<_, _, _> = call_builder.clone().clear_decoder();
/// let raw_result: Bytes = raw_builder.call().await?;
/// // Decode the raw bytes:
/// let decoded_result: Vec<DynYlmValue> = call_builder.decode_output(raw_result, false)?;
/// # Ok(())
/// # }
/// ```
///
/// [ylm]: base_ylm_types::ylm
#[derive(Clone)]
#[must_use = "call builders do nothing unless you `.call`, `.send`, or `.await` them"]
pub struct CallBuilder<T, P, D, N: Network = Ethereum> {
    request: N::TransactionRequest,
    block: BlockId,
    state: Option<StateOverride>,
    /// The provider.
    // NOTE: This is public due to usage in `ylm!`, please avoid changing it.
    pub provider: P,
    decoder: D,
    transport: PhantomData<T>,
}

// See [`ContractInstance`].
impl<T: Transport + Clone, P: Provider<T, N>, N: Network> DynCallBuilder<T, P, N> {
    pub(crate) fn new_dyn(provider: P, function: &Function, args: &[DynYlmValue]) -> Result<Self> {
        Ok(Self::new_inner_call(
            provider,
            function.abi_encode_input(args)?.into(),
            function.clone(),
        ))
    }

    /// Clears the decoder, returning a raw call builder.
    #[inline]
    pub fn clear_decoder(self) -> RawCallBuilder<T, P, N> {
        RawCallBuilder {
            request: self.request,
            block: self.block,
            state: self.state,
            provider: self.provider,
            decoder: (),
            transport: PhantomData,
        }
    }
}

#[doc(hidden)]
impl<'a, T: Transport + Clone, P: Provider<T, N>, C: YlmCall, N: Network>
    YlmCallBuilder<T, &'a P, C, N>
{
    // `ylm!` macro constructor, see `#[ylm(rpc)]`. Not public API.
    // NOTE: please avoid changing this function due to its use in the `ylm!` macro.
    pub fn new_sol(provider: &'a P, address: &IcanAddress, call: &C) -> Self {
        Self::new_inner_call(provider, call.abi_encode().into(), PhantomData::<C>).to(*address)
    }
}

impl<T: Transport + Clone, P: Provider<T, N>, C: YlmCall, N: Network> YlmCallBuilder<T, P, C, N> {
    /// Clears the decoder, returning a raw call builder.
    #[inline]
    pub fn clear_decoder(self) -> RawCallBuilder<T, P, N> {
        RawCallBuilder {
            request: self.request,
            block: self.block,
            state: self.state,
            provider: self.provider,
            decoder: (),
            transport: PhantomData,
        }
    }
}

impl<T: Transport + Clone, P: Provider<T, N>, N: Network> RawCallBuilder<T, P, N> {
    /// Sets the decoder to the provided [`YlmCall`].
    ///
    /// Converts the raw call builder into a ylm call builder.
    ///
    /// Note that generally you would want to instantiate a ylm call builder directly using the
    /// `ylm!` macro, but this method is provided for flexibility, for example to convert a raw
    /// deploy call builder into a ylm call builder.
    ///
    /// # Examples
    ///
    /// Decode a return value from a constructor:
    ///
    /// ```no_run
    /// # use base_ylm_types::ylm;
    /// ylm! {
    ///     // NOTE: This contract is not meant to be deployed on-chain, but rather
    ///     // used in a static call with its creation code as the call data.
    ///     #[ylm(rpc, bytecode = "34601457602a60e052600161010052604060e0f35b5f80fdfe")]
    ///     contract MyContract {
    ///         // The type returned by the constructor.
    ///         #[derive(Debug, PartialEq)]
    ///         struct MyStruct {
    ///             uint64 a;
    ///             bool b;
    ///         }
    ///
    ///         constructor() {
    ///             MyStruct memory s = MyStruct(42, true);
    ///             bytes memory returnData = abi.encode(s);
    ///             assembly {
    ///                 return(add(returnData, 0x20), mload(returnData))
    ///             }
    ///         }
    ///
    ///         // A shim that represents the return value of the constructor.
    ///         function constructorReturn() external view returns (MyStruct memory s);
    ///     }
    /// }
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # stringify!(
    /// let provider = ...;
    /// # );
    /// # let provider = atoms_provider::ProviderBuilder::new().on_anvil();
    /// let call_builder = MyContract::deploy_builder(&provider)
    ///     .with_ylm_decoder::<MyContract::constructorReturnCall>();
    /// let result = call_builder.call().await?;
    /// assert_eq!(result.s, MyContract::MyStruct { a: 42, b: true });
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn with_ylm_decoder<C: YlmCall>(self) -> YlmCallBuilder<T, P, C, N> {
        YlmCallBuilder {
            request: self.request,
            block: self.block,
            state: self.state,
            provider: self.provider,
            decoder: PhantomData::<C>,
            transport: PhantomData,
        }
    }
}

impl<T: Transport + Clone, P: Provider<T, N>, N: Network> RawCallBuilder<T, P, N> {
    /// Creates a new call builder with the provided provider and ABI encoded input.
    ///
    /// Will not decode the output of the call, meaning that [`call`](Self::call) will behave the
    /// same as [`call_raw`](Self::call_raw).
    #[inline]
    pub fn new_raw(provider: P, input: Bytes) -> Self {
        Self::new_inner_call(provider, input, ())
    }

    /// Creates a new call builder with the provided provider and contract deploy code.
    ///
    /// Will not decode the output of the call, meaning that [`call`](Self::call) will behave the
    /// same as [`call_raw`](Self::call_raw).
    // NOTE: please avoid changing this function due to its use in the `ylm!` macro.
    pub fn new_raw_deploy(provider: P, input: Bytes) -> Self {
        Self::new_inner_deploy(provider, input, ())
    }
}

impl<T: Transport + Clone, P: Provider<T, N>, D: CallDecoder, N: Network> CallBuilder<T, P, D, N> {
    fn new_inner_deploy(provider: P, input: Bytes, decoder: D) -> Self {
        Self {
            request: <N::TransactionRequest>::default().with_deploy_code(input),
            decoder,
            provider,
            block: BlockId::default(),
            state: None,
            transport: PhantomData,
        }
    }

    fn new_inner_call(provider: P, input: Bytes, decoder: D) -> Self {
        Self {
            request: <N::TransactionRequest>::default().with_input(input),
            decoder,
            provider,
            block: BlockId::default(),
            state: None,
            transport: PhantomData,
        }
    }

    /// Sets the `chain_id` field in the transaction to the provided value
    pub fn chain_id(mut self, chain_id: ChainId) -> Self {
        self.request.set_network_id(chain_id);
        self
    }

    /// Sets the `from` field in the transaction to the provided value.
    pub fn from(mut self, from: IcanAddress) -> Self {
        self.request.set_from(from);
        self
    }

    /// Sets the transaction request to the provided tx kind.
    pub fn kind(mut self, to: TxKind) -> Self {
        self.request.set_kind(to);
        self
    }

    /// Sets the `to` field in the transaction to the provided address.
    pub fn to(mut self, to: IcanAddress) -> Self {
        self.request.set_to(to);
        self
    }

    // /// Sets the `sidecar` field in the transaction to the provided value.
    // pub fn sidecar(mut self, blob_sidecar: BlobTransactionSidecar) -> Self {
    //     self.request.set_blob_sidecar(blob_sidecar);
    //     self
    // }

    /// Uses a Legacy transaction instead of an EIP-1559 one to execute the call
    pub fn legacy(self) -> Self {
        todo!()
    }

    /// Sets the `gas` field in the transaction to the provided value
    pub fn gas(mut self, gas: u128) -> Self {
        self.request.set_energy_limit(gas);
        self
    }

    /// Sets the `gas_price` field in the transaction to the provided value
    /// If the internal transaction is an EIP-1559 one, then it sets both
    /// `max_fee_per_gas` and `max_priority_fee_per_gas` to the same value
    pub fn gas_price(mut self, gas_price: u128) -> Self {
        self.request.set_energy_price(gas_price);
        self
    }

    /// Sets the `max_fee_per_gas` in the transaction to the provide value
    pub fn max_fee_per_gas(mut self, max_fee_per_gas: u128) -> Self {
        self.request.set_max_fee_per_gas(max_fee_per_gas);
        self
    }

    /// Sets the `max_priority_fee_per_gas` in the transaction to the provide value
    pub fn max_priority_fee_per_gas(mut self, max_priority_fee_per_gas: u128) -> Self {
        self.request.set_max_priority_fee_per_gas(max_priority_fee_per_gas);
        self
    }

    /// Sets the `max_fee_per_blob_gas` in the transaction to the provided value
    pub fn max_fee_per_blob_gas(mut self, max_fee_per_blob_gas: u128) -> Self {
        self.request.set_max_fee_per_blob_gas(max_fee_per_blob_gas);
        self
    }

    /// Sets the `access_list` in the transaction to the provided value
    pub fn access_list(mut self, access_list: AccessList) -> Self {
        // self.request.set_access_list(access_list);
        self
    }

    /// Sets the `value` field in the transaction to the provided value
    pub fn value(mut self, value: U256) -> Self {
        self.request.set_value(value);
        self
    }

    /// Sets the `nonce` field in the transaction to the provided value
    pub fn nonce(mut self, nonce: u64) -> Self {
        self.request.set_nonce(nonce);
        self
    }

    /// Applies a function to the internal transaction request.
    pub fn map<F>(mut self, f: F) -> Self
    where
        F: FnOnce(N::TransactionRequest) -> N::TransactionRequest,
    {
        self.request = f(self.request);
        self
    }

    /// Sets the `block` field for sending the tx to the chain
    pub const fn block(mut self, block: BlockId) -> Self {
        self.block = block;
        self
    }

    /// Sets the [state override set](https://geth.ethereum.org/docs/rpc/ns-eth#3-object---state-override-set).
    ///
    /// # Note
    ///
    /// Not all client implementations will support this as a parameter to `eth_call`.
    pub fn state(mut self, state: StateOverride) -> Self {
        self.state = Some(state);
        self
    }

    /// Returns the underlying transaction's ABI-encoded data.
    pub fn calldata(&self) -> &Bytes {
        self.request.input().expect("set in the constructor")
    }

    /// Returns the estimated gas cost for the underlying transaction to be executed
    pub async fn estimate_gas(&self) -> Result<u128> {
        self.provider.estimate_energy(&self.request, self.block).await.map_err(Into::into)
    }

    /// Queries the blockchain via an `eth_call` without submitting a transaction to the network.
    /// If [`state overrides`](Self::state) are set, they will be applied to the call.
    ///
    /// Returns the decoded the output by using the provided decoder.
    /// If this is not desired, use [`call_raw`](Self::call_raw) to get the raw output data.
    #[doc(alias = "eth_call")]
    #[doc(alias = "call_with_overrides")]
    pub fn call(&self) -> XcbCall<'_, '_, '_, D, T, N> {
        self.call_raw().with_decoder(&self.decoder)
    }

    /// Queries the blockchain via an `eth_call` without submitting a transaction to the network.
    /// If [`state overrides`](Self::state) are set, they will be applied to the call.
    ///
    /// Does not decode the output of the call, returning the raw output data instead.
    ///
    /// See [`call`](Self::call) for more information.
    pub fn call_raw(&self) -> XcbCall<'_, '_, '_, (), T, N> {
        let call = self.provider.call(&self.request).block(self.block);
        let call = match &self.state {
            Some(state) => call.overrides(state),
            None => call,
        };
        call.into()
    }

    /// Decodes the output of a contract function using the provided decoder.
    #[inline]
    pub fn decode_output(&self, data: Bytes, validate: bool) -> Result<D::CallOutput> {
        self.decoder.abi_decode_output(data, validate)
    }

    /// Broadcasts the underlying transaction to the network as a deployment transaction, returning
    /// the address of the deployed contract after the transaction has been confirmed.
    ///
    /// Returns an error if the transaction is not a deployment transaction, or if the contract
    /// address is not found in the deployment transaction’s receipt.
    ///
    /// For more fine-grained control over the deployment process, use [`send`](Self::send) instead.
    ///
    /// Note that the deployment address can be pre-calculated if the `from` address and `nonce` are
    /// known using [`calculate_create_address`](Self::calculate_create_address).
    pub async fn deploy(&self) -> Result<IcanAddress> {
        if !self.request.kind().is_some_and(|to| to.is_create()) {
            return Err(Error::NotADeploymentTransaction);
        }
        let pending_tx = self.send().await?;
        let receipt = pending_tx.get_receipt().await?;
        receipt.contract_address().ok_or(Error::ContractNotDeployed)
    }

    /// Broadcasts the underlying transaction to the network.
    ///
    /// Returns a builder for configuring the pending transaction watcher.
    /// See [`Provider::send_transaction`] for more information.
    pub async fn send(&self) -> Result<PendingTransactionBuilder<'_, T, N>> {
        Ok(self.provider.send_transaction(self.request.clone()).await?)
    }

    /// Calculates the address that will be created by the transaction, if any.
    ///
    /// Returns `None` if the transaction is not a contract creation (the `to` field is set), or if
    /// the `from` or `nonce` fields are not set.
    pub fn calculate_create_address(&self) -> Option<IcanAddress> {
        self.request.calculate_create_address()
    }
}

impl<T: Transport, P: Clone, D, N: Network> CallBuilder<T, &P, D, N> {
    /// Clones the provider and returns a new builder with the cloned provider.
    pub fn with_cloned_provider(self) -> CallBuilder<T, P, D, N> {
        CallBuilder {
            request: self.request,
            block: self.block,
            state: self.state,
            provider: self.provider.clone(),
            decoder: self.decoder,
            transport: PhantomData,
        }
    }
}

/// [`CallBuilder`] can be turned into a [`Future`] automatically with `.await`.
///
/// Defaults to calling [`CallBuilder::call`].
///
/// # Note
///
/// This requires `Self: 'static` due to a current limitation in the Rust type system, namely that
/// the associated future type, the returned future, must be a concrete type (`Box<dyn Future ...>`)
/// and cannot be an opaque type (`impl Future ...`) because `impl Trait` in this position is not
/// stable yet. See [rust-lang/rust#63063](https://github.com/rust-lang/rust/issues/63063).
impl<T, P, D, N> IntoFuture for CallBuilder<T, P, D, N>
where
    T: Transport + Clone,
    P: Provider<T, N>,
    D: CallDecoder + Send + Sync + Unpin,
    N: Network,
    Self: 'static,
{
    type Output = Result<D::CallOutput>;
    #[cfg(target_arch = "wasm32")]
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output>>>;
    #[cfg(not(target_arch = "wasm32"))]
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send>>;

    #[inline]
    fn into_future(self) -> Self::IntoFuture {
        #[allow(clippy::redundant_async_block)]
        Box::pin(async move { self.call().await })
    }
}

impl<T, P, D: CallDecoder, N: Network> std::fmt::Debug for CallBuilder<T, P, D, N> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CallBuilder")
            .field("request", &self.request)
            .field("block", &self.block)
            .field("state", &self.state)
            .field("decoder", &self.decoder.as_debug_field())
            .finish()
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;
    use atoms_network::Ethereum;
    use atoms_node_bindings::{Anvil, AnvilInstance};
    use atoms_provider::{
        layers::AnvilProvider, Provider, ProviderBuilder, ReqwestProvider, RootProvider,
        WalletProvider,
    };
    use atoms_rpc_client::RpcClient;
    use atoms_rpc_types::AccessListItem;
    use atoms_transport_http::Http;
    use base_primitives::{address, b256, bytes, cAddress, hex, utils::parse_units, Address, B256};
    use base_ylm_types::ylm;
    use reqwest::{Client, Url};

    #[test]
    fn empty_constructor() {
        ylm! {
            #[ylm(rpc, bytecode = "6942")]
            contract EmptyConstructor {
                constructor();
            }
        }

        let provider = ProviderBuilder::new().on_anvil();
        let call_builder = EmptyConstructor::deploy_builder(&provider);
        assert_eq!(*call_builder.calldata(), bytes!("6942"));
    }

    ylm! {
        // Solc: 0.8.24+commit.e11b9ed9.Linux.g++
        // Command: ylem a.ylm --bin --optimize --optimize-runs 1
        #[ylm(rpc, bytecode = "608060405234801561001057600080fd5b506040516101ac3803806101ac83398101604081905261002f91610045565b6000805460ff191691151591909117905561006c565b600060208284031215610056578081fd5b81518015158114610065578182fd5b9392505050565b6101318061007b6000396000f3fe6080604052348015600f57600080fd5b506004361060325760003560e01c80637fe45f4e146037578063b117ffa6146058575b600080fd5b60005460439060ff1681565b60405190151581526020015b60405180910390f35b6067606336600460a5565b6085565b604080516001600160b01b039093168352602083019190915201604f565b600080838360935760006096565b60015b90925060ff1690509250929050565b6000806040838503121560b6578182fd5b823591506020830135801515811460cb578182fd5b80915050925092905056fea26469706673582212200ea2bbf5c9db4fbfc298faab31c5289f6641dc875867ed916014b4510ec4853264736f6c637827302e382e342d646576656c6f702e323032322e382e32322b636f6d6d69742e61303164646338320058")]
        contract MyContract {
            bool public myState;

            constructor(bool myState_) {
                myState = myState_;
            }

            function doStuff(uint a, bool b) external pure returns(address c, bytes32 d) {
                return (address(uint176(a)), bytes32(uint256(b ? 1 : 0)));
            }
        }
    }

    ylm! {
        // Solc: 0.8.24+commit.e11b9ed9.Linux.g++
        // Command: ylem counter.ylm --bin --optimize --optimize-runs 1
        #[ylm(rpc, bytecode = "608060405234801561001057600080fd5b50610140806100206000396000f3fe6080604052348015600f57600080fd5b506004361060325760003560e01c80636d701db8146037578063bc1ecb8e146065575b600080fd5b6000546049906001600160801b031681565b6040516001600160801b03909116815260200160405180910390f35b606b606d565b005b6000805460019190819060899084906001600160801b031660af565b92506101000a8154816001600160801b0302191690836001600160801b03160217905550565b60006001600160801b0382811684821680830382111560dc57634b1f2ce360e01b84526011600452602484fd5b0194935050505056fea2646970667358221220008f79d64885516e4359c52ada4057f5591d397b06e28eedc1c1a6ad416e91d164736f6c637827302e382e342d646576656c6f702e323032322e382e32322b636f6d6d69742e61303164646338320058")]
        contract Counter {
            uint128 public counter;

            function increment() external {
                counter += 1;
            }
        }
    }

    /// Creates a new call_builder to test field modifications, taken from [call_encoding]
    #[allow(clippy::type_complexity)]
    fn build_call_builder() -> CallBuilder<
        Http<Client>,
        AnvilProvider<RootProvider<Http<Client>>, Http<Client>>,
        PhantomData<MyContract::doStuffCall>,
    > {
        let provider = ProviderBuilder::new().on_anvil();
        let contract = MyContract::new(IcanAddress::ZERO, provider);
        let call_builder = contract.doStuff(U256::ZERO, true).with_cloned_provider();
        call_builder
    }

    #[test]
    fn change_chain_id() {
        let call_builder = build_call_builder().chain_id(1337);
        assert_eq!(call_builder.request.network_id, 1337, "chain_id of request should be '1337'");
    }

    #[test]
    fn change_max_fee_per_gas() {
        let call_builder = build_call_builder().max_fee_per_gas(42);
        assert_eq!(
            call_builder.request.max_fee_per_gas.expect("max_fee_per_gas should be set"),
            42,
            "max_fee_per_gas of request should be '42'"
        );
    }

    #[test]
    fn change_max_priority_fee_per_gas() {
        let call_builder = build_call_builder().max_priority_fee_per_gas(45);
        assert_eq!(
            call_builder
                .request
                .max_priority_fee_per_gas
                .expect("max_priority_fee_per_gas should be set"),
            45,
            "max_priority_fee_per_gas of request should be '45'"
        );
    }

    #[test]
    fn change_max_fee_per_blob_gas() {
        let call_builder = build_call_builder().max_fee_per_blob_gas(50);
        assert_eq!(
            call_builder.request.max_fee_per_blob_gas.expect("max_fee_per_blob_gas should be set"),
            50,
            "max_fee_per_blob_gas of request should be '50'"
        );
    }

    #[test]
    fn change_access_list() {
        let access_list = AccessList::from(vec![AccessListItem {
            address: Address::ZERO,
            storage_keys: vec![B256::ZERO],
        }]);
        let call_builder = build_call_builder().access_list(access_list.clone());
        // assert_eq!(
        //     call_builder.request.access_list.expect("access_list should be set"),
        //     access_list,
        //     "Access list of the transaction should have been set to our access list"
        // )
    }

    #[test]
    fn call_encoding() {
        let provider = ProviderBuilder::new().on_anvil();
        let contract = MyContract::new(IcanAddress::ZERO, &&provider).with_cloned_provider();
        let call_builder = contract.doStuff(U256::ZERO, true).with_cloned_provider();
        assert_eq!(
            *call_builder.calldata(),
            bytes!(
                "b117ffa6"
                "0000000000000000000000000000000000000000000000000000000000000000"
                "0000000000000000000000000000000000000000000000000000000000000001"
            ),
        );
        // Box the future to assert its concrete output type.
        let _future: Box<dyn Future<Output = Result<MyContract::doStuffReturn>> + Send> =
            Box::new(async move { call_builder.call().await });
    }

    #[test]
    fn deploy_encoding() {
        let provider = ProviderBuilder::new().on_anvil();
        let bytecode = &MyContract::BYTECODE[..];
        let call_builder = MyContract::deploy_builder(&provider, false);
        assert_eq!(
            call_builder.calldata()[..],
            [
                bytecode,
                &hex!("0000000000000000000000000000000000000000000000000000000000000000")[..]
            ]
            .concat(),
        );
        let call_builder = MyContract::deploy_builder(&provider, true);
        assert_eq!(
            call_builder.calldata()[..],
            [
                bytecode,
                &hex!("0000000000000000000000000000000000000000000000000000000000000001")[..]
            ]
            .concat(),
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn deploy_and_call() {
        let provider = ProviderBuilder::new().with_recommended_fillers().on_anvil_with_signer();

        let expected_address = provider.default_signer_address().create(0);
        let my_contract = MyContract::deploy(provider, true).await.unwrap();
        assert_eq!(*my_contract.address(), expected_address);

        let my_state_builder = my_contract.myState();
        assert_eq!(my_state_builder.calldata()[..], MyContract::myStateCall {}.abi_encode(),);
        let result: MyContract::myStateReturn = my_state_builder.call().await.unwrap();
        assert!(result._0);

        let do_stuff_builder = my_contract.doStuff(U256::from(0x69), true);
        assert_eq!(
            do_stuff_builder.calldata()[..],
            MyContract::doStuffCall { a: U256::from(0x69), b: true }.abi_encode(),
        );
        let result: MyContract::doStuffReturn = do_stuff_builder.call().await.unwrap();
        assert_eq!(result.c, cAddress!("00000000000000000000000000000000000000000069"));
        assert_eq!(
            result.d,
            b256!("0000000000000000000000000000000000000000000000000000000000000001"),
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn deploy_and_call_with_priority() {
        let provider = ProviderBuilder::new().with_recommended_fillers().on_anvil_with_signer();
        let counter_contract = Counter::deploy(provider.clone()).await.unwrap();
        let receipt = counter_contract
            .increment()
            .gas_price(1000)
            .send()
            .await
            .expect("Could not send transaction")
            .get_receipt()
            .await
            .expect("Could not get the receipt");
        let transaction_hash = receipt.transaction_hash;
        let transaction = provider
            .get_transaction_by_hash(transaction_hash)
            .await
            .expect("failed to fetch tx")
            .expect("tx not included");
        assert_eq!(
            transaction.energy_price.expect("energy_price of the transaction should be set"),
            1000,
            "energy_price of the transaction should be set to the right value"
        );
    }
}
