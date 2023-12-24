mod eip2718;
pub use eip2718::{Decodable2718, Eip2718Envelope, Eip2718Error, Encodable2718};

mod signed;
pub use signed::Signed;

use alloy_primitives::{Bytes, ChainId, Signature, B256, U256};
use alloy_rlp::{BufMut, Encodable};

/// Represents a transaction.
pub trait Transaction: Encodable + Send + Sync + 'static {
    /// Convert to a signed transaction by adding a signature and computing the
    /// hash.
    fn into_signed(self, signature: Signature) -> Signed<Self>
    where
        Self: Sized;

    /// Encode with a signature. This encoding is usually RLP, but may be
    /// different for future EIP-2718 transaction types.
    fn encode_signed(&self, signature: &Signature, out: &mut dyn BufMut);

    /// Decode a signed transaction. This decoding is usually RLP, but may be
    /// different for future EIP-2718 transaction types.
    ///
    /// This MUST be the inverse of [`Transaction::encode_signed`].
    fn decode_signed(buf: &mut &[u8]) -> alloy_rlp::Result<Signed<Self>>
    where
        Self: Sized;

    /// Calculate the signing hash for the transaction.
    fn signature_hash(&self) -> B256;

    /// Get `data`.
    fn input(&self) -> &[u8];
    /// Mut getter for
    fn input_mut(&mut self) -> &mut Bytes;
    /// Set `data`.
    fn set_input(&mut self, data: Bytes);

    /// Get `value`.
    fn value(&self) -> U256;
    /// Set `value`.
    fn set_value(&mut self, value: U256);

    /// Get `chain_id`.
    fn chain_id(&self) -> Option<ChainId>;
    /// Set `chain_id`.
    fn set_chain_id(&mut self, chain_id: ChainId);

    /// Get `nonce`.
    fn nonce(&self) -> u64;
    /// Set `nonce`.
    fn set_nonce(&mut self, nonce: u64);

    /// Get `gas_limit`.
    fn gas_limit(&self) -> u64;
    /// Set `gas_limit`.
    fn set_gas_limit(&mut self, limit: u64);
}

/// Captures getters and setters common across EIP-1559 transactions across all networks
pub trait Eip1559Transaction: Transaction {
    /// Get `max_priority_fee_per_gas`.
    fn max_priority_fee_per_gas(&self) -> U256;
    /// Set `max_priority_fee_per_gas`.
    fn set_max_priority_fee_per_gas(&mut self, max_priority_fee_per_gas: U256);

    /// Get `max_fee_per_gas`.
    fn max_fee_per_gas(&self) -> U256;
    /// Set `max_fee_per_gas`.
    fn set_max_fee_per_gas(&mut self, max_fee_per_gas: U256);

    /// Get `max_tip` (an alias for `max_priority_fee_per_gas`).
    fn max_tip(&self) -> U256 {
        self.max_priority_fee_per_gas()
    }

    /// Set max tip, (an alias for `max_priority_fee_per_gas`).
    fn set_max_tip(&mut self, max_tip: U256) {
        self.set_max_priority_fee_per_gas(max_tip)
    }
}
