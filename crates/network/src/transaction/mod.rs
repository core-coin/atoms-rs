use alloy_primitives::{keccak256, Bytes, ChainId, Signature, B256, U256};
use alloy_rlp::BufMut;

mod common;
pub use common::TxKind;

mod signed;
pub use signed::Signed;

/// Transaction-like objects signable with a specific signature type.
pub trait Signable<Sig = Signature>: Transaction {
    /// RLP-encodes the transaction for signing.
    fn encode_for_signing(&self, out: &mut dyn alloy_rlp::BufMut);

    /// Outputs the length of the signature RLP encoding for the transaction.
    fn payload_len_for_signature(&self) -> usize;

    /// RLP-encodes the transaction for signing it. Used to calculate `signature_hash`.
    ///
    /// See [`Transaction::encode_for_signing`].
    fn encoded_for_signing(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.payload_len_for_signature());
        self.encode_for_signing(&mut buf);
        buf
    }

    /// Calculate the signing hash for the transaction.
    fn signature_hash(&self) -> B256 {
        keccak256(self.encoded_for_signing())
    }

    /// Convert to a signed transaction by adding a signature and computing the
    /// hash.
    fn into_signed(self, signature: Sig) -> Signed<Self, Sig>
    where
        Self: Sized;

    /// Encode with a signature. This encoding is usually RLP, but may be
    /// different for future EIP-2718 transaction types.
    fn encode_signed(&self, signature: &Sig, out: &mut dyn BufMut);

    /// Decode a signed transaction. This decoding is usually RLP, but may be
    /// different for future EIP-2718 transaction types.
    ///
    /// This MUST be the inverse of [`Transaction::encode_signed`].
    fn decode_signed(buf: &mut &[u8]) -> alloy_rlp::Result<Signed<Self, Sig>>
    where
        Self: Sized;
}

/// Represents a minimal EVM transaction.
pub trait Transaction: std::any::Any + Send + Sync + 'static {
    /// Get `data`.
    fn input(&self) -> &[u8];
    /// Get `data`.
    fn input_mut(&mut self) -> &mut Bytes;
    /// Set `data`.
    fn set_input(&mut self, data: Bytes);

    /// Get `to`.
    fn to(&self) -> TxKind;
    /// Set `to`.
    fn set_to(&mut self, to: TxKind);

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

    /// Get `gas_price`.
    fn gas_price(&self) -> Option<U256>;
    /// Set `gas_price`.
    fn set_gas_price(&mut self, price: U256);
}

// TODO: Remove in favor of dyn trait upcasting (TBD, see https://github.com/rust-lang/rust/issues/65991#issuecomment-1903120162)
#[doc(hidden)]
impl<S: 'static> dyn Signable<S> {
    pub fn __downcast_ref<T: std::any::Any>(&self) -> Option<&T> {
        if std::any::Any::type_id(self) == std::any::TypeId::of::<T>() {
            unsafe { Some(&*(self as *const _ as *const T)) }
        } else {
            None
        }
    }
}

/// Captures getters and setters common across EIP-1559 transactions across all networks
pub trait Eip1559Transaction: Transaction {
    /// Get `max_priority_fee_per_gas`.
    #[doc(alias = "max_tip")]
    fn max_priority_fee_per_gas(&self) -> U256;
    /// Set `max_priority_fee_per_gas`.
    #[doc(alias = "set_max_tip")]
    fn set_max_priority_fee_per_gas(&mut self, max_priority_fee_per_gas: U256);

    /// Get `max_fee_per_gas`.
    fn max_fee_per_gas(&self) -> U256;
    /// Set `max_fee_per_gas`.
    fn set_max_fee_per_gas(&mut self, max_fee_per_gas: U256);
}
