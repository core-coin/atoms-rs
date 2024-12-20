# atoms-signer

Core signer abstraction.

You can implement the [`Signer`][Signer] trait to extend functionality to other signers
such as Hardware Security Modules, KMS etc. See [its documentation][Signer] for more.

Signer implementation in Alloy:
- [K256 private key](../signer-wallet/src/private_key.rs)
- [YubiHSM2](../signer-wallet/src/yubi.rs)
- [Ledger](../signer-ledger/)
- [Trezor](../signer-trezor/)
- [AWS KMS](../signer-aws/)
- [GCP KMS](../signer-gcp/)

<!-- TODO: docs.rs -->
[Signer]: https://base-rs.github.io/alloy/atoms_signer/trait.Signer.html

## Examples

Sign an Core prefixed message ([EIP-712](https://eips.ethereum.org/EIPS/eip-712)):

```rust
use atoms_signer::{Signer, SignerSync};

// Instantiate a signer.
let signer = atoms_signer_wallet::LocalWallet::random(1);

// Sign a message.
let message = "Some data";
let signature = signer.sign_message_sync(message.as_bytes())?;

// Recover the signer from the message.
let recovered = signature.recover_address_from_msg(message, 1)?;
assert_eq!(recovered, signer.address());
# Ok::<_, Box<dyn std::error::Error>>(())
```

Sign a transaction:

```rust
use atoms_consensus::TxLegacy;
use base_primitives::{U256, cAddress, bytes};
use atoms_signer::{Signer, SignerSync};
use atoms_network::{TxSignerSync};
use libgoldilocks::SigningKey;

// Instantiate a signer.
let signer = atoms_signer_wallet::LocalWallet::from_signing_key(SigningKey::from_str("dcf2cbdd171a21c480aa7f53d77f31bb102282b3ff099c78e3118b37348c72f7dcf2cbdd171a21c480aa7f53d77f31bb102282b3ff099c78e3"), 1);

// Create a transaction.
let mut tx = TxLegacy {
    to: cAddress!("0000d8dA6BF26964aF9D7eEd9e03E53415D37aA96045").into(),
    value: U256::from(1_000_000_000),
    energy_limit: 2_000_000,
    nonce: 0,
    energy_price: 21_000_000_000,
    input: bytes!(),
    network_id: 1,
};

// Sign it.
let signature = signer.sign_transaction_sync(&mut tx)?;
# Ok::<_, Box<dyn std::error::Error>>(())
```
