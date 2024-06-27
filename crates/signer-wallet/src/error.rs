use alloy_primitives::hex;
use thiserror::Error;

/// Error thrown by [`Wallet`](crate::Wallet).
#[derive(Debug, Error)]
pub enum WalletError {
    /// [`hex`](mod@hex) error.
    #[error(transparent)]
    HexError(#[from] hex::FromHexError),
    /// [`std::io`] error.
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    /// Libgoldilocks error.
    #[error(transparent)]
    LibgoldilockError(#[from] libgoldilocks::errors::LibgoldilockErrors),

    /// [`coins_bip32`] error.
    #[error(transparent)]
    #[cfg(feature = "mnemonic")]
    Bip32Error(#[from] coins_bip32::Bip32Error),
    /// [`coins_bip39`] error.
    #[error(transparent)]
    #[cfg(feature = "mnemonic")]
    Bip39Error(#[from] coins_bip39::MnemonicError),
    /// [`MnemonicBuilder`](super::mnemonic::MnemonicBuilder) error.
    // #[error(transparent)]
    // #[cfg(feature = "mnemonic")]
    // MnemonicBuilderError(#[from] super::mnemonic::MnemonicBuilderError),

    /// [`xcb_keystore`] error.
    #[cfg(feature = "keystore")]
    #[error(transparent)]
    EthKeystoreError(#[from] xcb_keystore::KeystoreError),
}
