use crate::{CallBuilder, Event, Interface, Result};
use atoms_rpc_types::Filter;
use atoms_transport::Transport;
use base_dyn_abi::DynYlmValue;
use base_json_abi::{Function, JsonAbi};
use atoms_network::{Ethereum, Network};
use base_primitives::{IcanAddress, Selector};
use atoms_provider::Provider;
use base_ylm_types::YlmEvent;
use std::marker::PhantomData;

/// A handle to an Ethereum contract at a specific address.
///
/// A contract is an abstraction of an executable program on Ethereum. Every deployed contract has
/// an address, which is used to connect to it so that it may receive messages (transactions).
#[derive(Clone)]
pub struct ContractInstance<T, P, N = Ethereum> {
    address: IcanAddress,
    provider: P,
    interface: Interface,
    transport: PhantomData<T>,
    network: PhantomData<N>,
}

impl<T, P, N> ContractInstance<T, P, N> {
    /// Creates a new contract from the provided address, provider, and interface.
    #[inline]
    pub const fn new(address: IcanAddress, provider: P, interface: Interface) -> Self {
        Self { address, provider, interface, transport: PhantomData, network: PhantomData }
    }

    /// Returns a reference to the contract's address.
    #[inline]
    pub const fn address(&self) -> &IcanAddress {
        &self.address
    }

    /// Sets the contract's address.
    #[inline]
    pub fn set_address(&mut self, address: IcanAddress) {
        self.address = address;
    }

    /// Returns a new contract instance at `address`.
    #[inline]
    pub fn at(mut self, address: IcanAddress) -> ContractInstance<T, P, N> {
        self.set_address(address);
        self
    }

    /// Returns a reference to the contract's ABI.
    #[inline]
    pub const fn abi(&self) -> &JsonAbi {
        self.interface.abi()
    }

    /// Returns a reference to the contract's provider.
    #[inline]
    pub const fn provider(&self) -> &P {
        &self.provider
    }
}

impl<T, P: Clone, N> ContractInstance<T, &P, N> {
    /// Clones the provider and returns a new contract instance with the cloned provider.
    #[inline]
    pub fn with_cloned_provider(self) -> ContractInstance<T, P, N> {
        ContractInstance {
            address: self.address,
            provider: self.provider.clone(),
            interface: self.interface,
            transport: PhantomData,
            network: PhantomData,
        }
    }
}

impl<T: Transport + Clone, P: Provider<T, N>, N: Network> ContractInstance<T, P, N> {
    /// Returns a transaction builder for the provided function name.
    ///
    /// If there are multiple functions with the same name due to overloading, consider using
    /// the [`ContractInstance::function_from_selector`] method instead, since this will use the
    /// first match.
    pub fn function(
        &self,
        name: &str,
        args: &[DynYlmValue],
    ) -> Result<CallBuilder<T, &P, Function, N>> {
        let function = self.interface.get_from_name(name)?;
        CallBuilder::new_dyn(&self.provider, function, args)
    }

    /// Returns a transaction builder for the provided function selector.
    pub fn function_from_selector(
        &self,
        selector: &Selector,
        args: &[DynYlmValue],
    ) -> Result<CallBuilder<T, &P, Function, N>> {
        let function = self.interface.get_from_selector(selector)?;
        CallBuilder::new_dyn(&self.provider, function, args)
    }

    /// Returns an [`Event`] builder with the provided filter.
    pub fn event<E: YlmEvent>(&self, filter: Filter) -> Event<T, &P, E, N> {
        Event::new(&self.provider, filter)
    }
}

impl<T, P, N> std::ops::Deref for ContractInstance<T, P, N> {
    type Target = Interface;

    fn deref(&self) -> &Self::Target {
        &self.interface
    }
}

impl<T, P, N> std::fmt::Debug for ContractInstance<T, P, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContractInstance").field("address", &self.address).finish()
    }
}
