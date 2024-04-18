use crate::{
    fillers::{FillProvider, JoinFill, SignerFiller, TxFiller},
    Provider,
};
use alloy_network::{Ethereum, Network, NetworkSigner};
use alloy_primitives::Address;
use alloy_transport::Transport;

/// Trait for Providers, Fill stacks, etc, which contain [`NetworkSigner`].
pub trait WalletProvider<N: Network = Ethereum> {
    /// The underlying [`NetworkSigner`] type contained in this stack.
    type Signer: NetworkSigner<N>;

    /// Get a reference to the underlying signer.
    fn signer(&self) -> &Self::Signer;

    /// Get the default signer address.
    fn default_signer(&self) -> Address {
        self.signer().default_signer()
    }

    /// Check if the signer can sign for the given address.
    fn is_signer_for(&self, address: &Address) -> bool {
        self.signer().is_signer_for(address)
    }

    /// Get an iterator of all signer addresses.
    fn signers(&self) -> impl Iterator<Item = Address> {
        self.signer().signers()
    }
}

impl<S, N> WalletProvider<N> for SignerFiller<S>
where
    S: NetworkSigner<N> + Clone,
    N: Network,
{
    type Signer = S;

    #[inline(always)]
    fn signer(&self) -> &Self::Signer {
        self.as_ref()
    }
}

impl<L, R, N> WalletProvider<N> for JoinFill<L, R>
where
    R: WalletProvider<N>,
    N: Network,
{
    type Signer = R::Signer;

    #[inline(always)]
    fn signer(&self) -> &Self::Signer {
        self.right().signer()
    }
}

impl<F, P, T, N> WalletProvider<N> for FillProvider<F, P, T, N>
where
    F: TxFiller<N> + WalletProvider<N>,
    P: Provider<T, N>,
    T: Transport + Clone,
    N: Network,
{
    type Signer = F::Signer;

    #[inline(always)]
    fn signer(&self) -> &Self::Signer {
        self.filler.signer()
    }
}
