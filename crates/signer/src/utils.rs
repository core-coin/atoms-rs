//! Utility functions for working with Core signatures.

use base_primitives::IcanAddress;
use libgoldilocks::{SigningKey, VerifyingKey};

/// Converts an ECDSA private key to its corresponding Core Address.
#[inline]
pub fn secret_key_to_address(secret_key: &SigningKey, network_id: u64) -> IcanAddress {
    IcanAddress::from_private_key(secret_key, network_id)
}

/// Converts an ECDSA public key to its corresponding Core address.
#[inline]
pub fn public_key_to_address(pubkey: &VerifyingKey, network_id: u64) -> IcanAddress {
    IcanAddress::from_public_key(pubkey, network_id)
}

/// Convert a raw, uncompressed public key to its corresponding Core address.
///
/// ### Warning
///
/// This method **does not** verify that the public key is valid. It is the
/// caller's responsibility to pass a valid public key. Passing an invalid
/// public key will produce an unspendable output.
///
/// # Panics
///
/// This function panics if the input is not **exactly** 64 bytes.
#[inline]
#[track_caller]
pub fn raw_public_key_to_address(pubkey: &[u8], network_id: u64) -> IcanAddress {
    IcanAddress::from_raw_public_key(pubkey, network_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use base_primitives::hex;

    // Only tests for correctness, no edge cases. Uses examples from https://docs.ethers.org/v5/api/utils/address/#utils-computeAddress
    #[test]
    fn test_public_key_to_address() {
        let addr = "cb82a5fd22b9bee8b8ab877c86e0a2c21765e1d5bfc5".parse::<IcanAddress>().unwrap();

        let pubkey = VerifyingKey::from_str(
            "315484db568379ce94f9c894e3e6e4c7ee216676b713ca892d9b26746ae902a772e217a6a8bb493ce2bb313cf0cb66e76765d4c45ec6b68600");
        assert_eq!(public_key_to_address(&pubkey, 1), addr);
    }

    #[test]
    fn test_raw_public_key_to_address() {
        let addr = "cb82a5fd22b9bee8b8ab877c86e0a2c21765e1d5bfc5".parse::<IcanAddress>().unwrap();

        let pubkey_bytes = hex::decode("315484db568379ce94f9c894e3e6e4c7ee216676b713ca892d9b26746ae902a772e217a6a8bb493ce2bb313cf0cb66e76765d4c45ec6b68600").unwrap();

        assert_eq!(raw_public_key_to_address(&pubkey_bytes, 1), addr);
    }

    #[test]
    #[should_panic]
    fn test_raw_public_key_to_address_panics() {
        raw_public_key_to_address(&[], 1);
    }
}
