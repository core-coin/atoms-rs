#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/alloy-rs/core/main/assets/alloy.jpg",
    html_favicon_url = "https://raw.githubusercontent.com/alloy-rs/core/main/assets/favicon.ico"
)]
#![warn(
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    clippy::missing_const_for_fn,
    rustdoc::all
)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![deny(unused_must_use, rust_2018_idioms)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

use alloy_consensus::SignableTransaction;
use alloy_network::{TxSigner, TxSignerSync};
use alloy_primitives::{ChainId, IcanAddress, Signature, B256};
use alloy_signer::{sign_transaction_with_network_id, Error, Result, Signer, SignerSync};
use async_trait::async_trait;
use libgoldilocks::{PrehashSigner, SigningKey};
use std::fmt;

mod error;
pub use error::WalletError;

// #[cfg(feature = "mnemonic")]
// mod mnemonic;
// #[cfg(feature = "mnemonic")]
// pub use mnemonic::MnemonicBuilder;

mod private_key;

#[cfg(feature = "yubihsm")]
mod yubi;

#[cfg(feature = "yubihsm")]
pub use yubihsm;

#[cfg(feature = "mnemonic")]
pub use coins_bip39;

/// A wallet instantiated with a locally stored private key
pub type LocalWallet = Wallet<SigningKey>;

/// A wallet instantiated with a YubiHSM
#[cfg(feature = "yubihsm")]
pub type YubiWallet = Wallet<yubihsm::ecdsa::Signer<k256::Secp256k1>>;

/// An Ethereum private-public key pair which can be used for signing messages.
///
/// # Examples
///
/// ## Signing and Verifying a message
///
/// The wallet can be used to produce ECDSA [`Signature`] objects, which can be
/// then verified. Note that this uses
/// [`eip191_hash_message`](alloy_primitives::eip191_hash_message) under the hood which will
/// prefix the message being hashed with the `Ethereum Signed Message` domain separator.
///
/// ```
/// use alloy_signer::{Signer, SignerSync};
///
/// let wallet = alloy_signer_wallet::LocalWallet::random(1);
///
/// // Optionally, the wallet's chain id can be set, in order to use EIP-155
/// // replay protection with different chains
/// let wallet = wallet.with_network_id(Some(1337));
///
/// // The wallet can be used to sign messages
/// let message = b"hello";
/// let signature = wallet.sign_message_sync(message)?;
///
/// // LocalWallet is clonable:
/// let wallet_clone = wallet.clone();
/// let signature2 = wallet_clone.sign_message_sync(message)?;
/// assert_eq!(signature, signature2);
/// # Ok::<_, Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone)]
pub struct Wallet<D> {
    /// The wallet's private key.
    pub(crate) signer: D,
    /// The wallet's address.
    pub(crate) address: IcanAddress,
    /// The wallet's network ID.
    pub(crate) network_id: Option<ChainId>,
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<D: PrehashSigner<Signature> + Send + Sync> Signer for Wallet<D> {
    #[inline]
    async fn sign_hash(&self, hash: &B256) -> Result<Signature> {
        self.sign_hash_sync(hash)
    }

    #[inline]
    fn address(&self) -> IcanAddress {
        self.address
    }

    #[inline]
    fn network_id(&self) -> Option<ChainId> {
        self.network_id
    }

    #[inline]
    fn set_network_id(&mut self, network_id: Option<ChainId>) {
        self.network_id = network_id;
    }
}

impl<D: PrehashSigner<Signature>> SignerSync for Wallet<D> {
    #[inline]
    fn sign_hash_sync(&self, hash: &B256) -> Result<Signature> {
        let sig = self.signer.sign_prehash(hash.as_ref()).map_err(|e| Error::Other(Box::new(e)))?;
        Ok(sig)
    }

    #[inline]
    fn network_id_sync(&self) -> Option<ChainId> {
        self.network_id
    }
}

impl<D: PrehashSigner<Signature>> Wallet<D> {
    /// Construct a new wallet with an external [`PrehashSigner`].
    #[inline]
    pub const fn new_with_signer(
        signer: D,
        address: IcanAddress,
        network_id: Option<ChainId>,
    ) -> Self {
        Wallet { signer, address, network_id }
    }

    /// Returns this wallet's signer.
    #[inline]
    pub const fn signer(&self) -> &D {
        &self.signer
    }

    /// Consumes this wallet and returns its signer.
    #[inline]
    pub fn into_signer(self) -> D {
        self.signer
    }

    /// Returns this wallet's chain ID.
    #[inline]
    pub const fn address(&self) -> IcanAddress {
        self.address
    }

    /// Returns this wallet's chain ID.
    #[inline]
    pub const fn network_id(&self) -> Option<ChainId> {
        self.network_id
    }
}

// do not log the signer
impl<D: PrehashSigner<Signature>> fmt::Debug for Wallet<D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Wallet")
            .field("address", &self.address)
            .field("chain_id", &self.network_id)
            .finish()
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<D> TxSigner<Signature> for Wallet<D>
where
    D: PrehashSigner<Signature> + Send + Sync,
{
    fn address(&self) -> IcanAddress {
        self.address
    }

    async fn sign_transaction(
        &self,
        tx: &mut dyn SignableTransaction<Signature>,
    ) -> alloy_signer::Result<Signature> {
        sign_transaction_with_network_id!(self, tx, self.sign_hash_sync(&tx.signature_hash()))
    }
}

impl<D> TxSignerSync<Signature> for Wallet<D>
where
    D: PrehashSigner<Signature>,
{
    fn address(&self) -> IcanAddress {
        self.address
    }

    fn sign_transaction_sync(
        &self,
        tx: &mut dyn SignableTransaction<Signature>,
    ) -> alloy_signer::Result<Signature> {
        sign_transaction_with_network_id!(self, tx, self.sign_hash_sync(&tx.signature_hash()))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use alloy_consensus::TxLegacy;
    use alloy_primitives::{cAddress, U256};

    #[tokio::test]
    async fn signs_tx() {
        async fn sign_tx_test(tx: &mut TxLegacy, chain_id: Option<ChainId>) -> Result<Signature> {
            let mut before = tx.clone();
            let sig = sign_dyn_tx_test(tx, chain_id).await?;
            if let Some(chain_id) = chain_id {
                assert_eq!(tx.network_id, Some(chain_id), "chain ID was not set");
                before.network_id = Some(chain_id);
            }
            assert_eq!(*tx, before);
            Ok(sig)
        }

        async fn sign_dyn_tx_test(
            tx: &mut dyn SignableTransaction<Signature>,
            chain_id: Option<ChainId>,
        ) -> Result<Signature> {
            let mut wallet: LocalWallet = LocalWallet::from_signing_key(
                SigningKey::from_str(
                    "7d6231471b5dbb6204fe5129617082792ae468d01a3f3623184c0883a69102937d6231471b5dbb6204fe5129617082792ae468d01a3f362318",
                ),
                1,
            );
            wallet.set_network_id(chain_id);

            let sig = wallet.sign_transaction_sync(tx)?;
            let sighash = tx.signature_hash();
            assert_eq!(sig.recover_address_from_prehash(&sighash, 1).unwrap(), wallet.address());

            let sig_async = wallet.sign_transaction(tx).await.unwrap();
            assert_eq!(sig_async, sig);

            Ok(sig)
        }

        // retrieved test vector from:
        // https://web3js.readthedocs.io/en/v1.2.0/web3-eth-accounts.html#eth-accounts-signtransaction
        let mut tx = TxLegacy {
            to: cAddress!("0000F0109fC8DF283027b6285cc889F5aA624EaC1F55").into(),
            value: U256::from(1_000_000_000),
            energy_limit: 2_000_000,
            nonce: 0,
            energy_price: 21_000_000_000,
            input: Default::default(),
            network_id: None,
        };
        let sig_none = sign_tx_test(&mut tx, None).await.unwrap();

        tx.network_id = Some(1);
        let sig_1 = sign_tx_test(&mut tx, None).await.unwrap();
        let expected = "0xba72dc3eb2a1bfe2539fb75f8bf42c7dc961afc4617dce529dfbd37ca7af5cab0b2ad96c7667b5f17bbd543d1e1b3d2d37b50fa9ec2e9a31805ead4189af9938fdc3a9038dd681e0ee3c9697c4caa86a3ab9f7da97a0b991ba7ed606ce9a678edc218e1ca621ac3bf67ccd3930d9b56b1a00c451ec0deec4f6ea947aeed0b53b6b3a31b2c98195652dbfebe5551ca44cd31dc99ed4e35beb49e97621f4513637ba768911282e695edab180".parse().unwrap();
        assert_eq!(sig_1, expected);
        assert_ne!(sig_1, sig_none);

        tx.network_id = Some(2);
        let sig_2 = sign_tx_test(&mut tx, None).await.unwrap();
        assert_ne!(sig_2, sig_1);
        assert_ne!(sig_2, sig_none);

        // Sets chain ID.
        tx.network_id = None;
        let sig_none_none = sign_tx_test(&mut tx, None).await.unwrap();
        assert_eq!(sig_none_none, sig_none);

        tx.network_id = None;
        let sig_none_1 = sign_tx_test(&mut tx, Some(1)).await.unwrap();
        assert_eq!(sig_none_1, sig_1);

        tx.network_id = None;
        let sig_none_2 = sign_tx_test(&mut tx, Some(2)).await.unwrap();
        assert_eq!(sig_none_2, sig_2);

        // Errors on mismatch.
        tx.network_id = Some(2);
        let error = sign_tx_test(&mut tx, Some(1)).await.unwrap_err();
        let expected_error = alloy_signer::Error::TransactionNetworkIdMismatch { signer: 1, tx: 2 };
        assert_eq!(error.to_string(), expected_error.to_string());
    }
}
