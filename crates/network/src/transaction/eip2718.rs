use alloy_primitives::keccak256;
use alloy_rlp::{BufMut, Header};

use crate::Sealed;

// https://eips.ethereum.org/EIPS/eip-2718#transactiontype-only-goes-up-to-0x7f
const TX_TYPE_BYTE_MAX: u8 = 0x7f;

/// [EIP-2718] decoding errors.
///
/// [EIP-2718]: https://eips.ethereum.org/EIPS/eip-2718
#[derive(thiserror::Error, Debug, Clone, Copy)]
pub enum Eip2718Error {
    /// Rlp error from [`alloy_rlp`].
    #[error(transparent)]
    RlpError(#[from] alloy_rlp::Error),
    /// Got an unexpected type flag while decoding.
    #[error("Unexpected type flag. Got {0}.")]
    UnexpectedType(u8),
}

/// Decoding trait for [EIP-2718] envelopes.
///
/// [EIP-2718]: https://eips.ethereum.org/EIPS/eip-2718
pub trait Decodable2718: Sized {
    /// Extract the type byte from the buffer, if any. The type byte is the
    /// first byte, provided that that first byte is 0x7f or lower.
    fn extract_type_byte(buf: &mut &[u8]) -> Option<u8> {
        buf.first().copied().filter(|b| *b <= TX_TYPE_BYTE_MAX)
    }

    /// Decode the appropriate variant, based on the type flag. This function
    /// is invoked by [`Self::decode`] with the type byte, and the tail of the
    /// buffer.
    ///
    /// ## Note
    ///
    /// This should be a simple match block that invokes an inner type's
    /// RLP decoder.
    fn typed_decode(ty: u8, buf: &mut &[u8]) -> Result<Self, Eip2718Error>;

    /// Decode the default variant. This function is invoked by
    /// [`Self::decode`] when no type byte can be extracted.
    fn fallback_decode(buf: &mut &[u8]) -> Result<Self, Eip2718Error>;

    /// Decode an EIP-2718 transaction into a concrete instance
    fn decode_2718(buf: &mut &[u8]) -> Result<Self, Eip2718Error> {
        Self::extract_type_byte(buf)
            .map(|ty| Self::typed_decode(ty, &mut &buf[1..]))
            .unwrap_or_else(|| Self::fallback_decode(buf))
    }

    /// Decode an EIP-2718 transaction in the network format.
    fn network_decode(buf: &mut &[u8]) -> Result<Self, Eip2718Error> {
        let h_decode = &mut *buf;
        let h = Header::decode(h_decode)?;

        if h.list {
            return Self::fallback_decode(buf);
        } else {
            *buf = h_decode;
        }

        let pre_len = buf.len();
        if pre_len == 0 || pre_len < h.payload_length {
            return Err(alloy_rlp::Error::InputTooShort.into());
        }
        let ty = buf[0];
        let buf = &mut &buf[1..];
        let tx = Self::typed_decode(ty, buf)?;

        if buf.len() != pre_len - h.payload_length {
            return Err(alloy_rlp::Error::UnexpectedLength.into());
        }

        Ok(tx)
    }
}

/// Encoding trait for [EIP-2718] envelopes.
///
/// [EIP-2718]: https://eips.ethereum.org/EIPS/eip-2718
pub trait Encodable2718: Sized + Send + Sync + 'static {
    /// Return the type flag (if any).
    ///
    /// This should return `None` for the default (legacy) variant of the
    /// envelope.
    fn type_flag(&self) -> Option<u8>;

    /// The length of the 2718 encoded envelope. This is the length of the type
    /// flag + the length of the inner transaction RLP.
    fn encode_2718_len(&self) -> usize;

    /// Encode the transaction according to [EIP-2718] rules. First a 1-byte
    /// type flag in the range 0x0-0x7f, then the body of the transaction.
    ///
    /// This implementation uses RLP for the transaction body. Non-standard
    /// users can override this to use some other serialization scheme.
    ///
    /// [EIP-2718]: https://eips.ethereum.org/EIPS/eip-2718
    fn encode_2718(&self, out: &mut dyn BufMut);

    /// Encode the transaction according to [EIP-2718] rules. First a 1-byte
    /// type flag in the range 0x0-0x7f, then the body of the transaction.
    ///
    /// This is a convenience method for encoding into a vec, and returning the
    /// vec.
    fn encoded_2718(&self) -> Vec<u8> {
        let mut out = vec![];
        self.encode_2718(&mut out);
        out
    }

    /// Seal the encodable, by encoding and hashing it.
    fn seal(self) -> Sealed<Self> {
        let buf = self.encoded_2718();
        let hash = keccak256(buf);
        Sealed::new_unchecked(self, hash)
    }

    /// Return the network encoding. This is the RLP encoding of the bytestring
    /// of the 2718 encoding.
    fn network_encode(&self, out: &mut dyn BufMut) {
        Header { list: false, payload_length: self.encode_2718_len() }.encode(out);
        self.encode_2718(out);
    }
}

/// An [EIP-2718] envelope, blanket implemented for types that impl
/// [`Encodable2718`] and [`Decodable2718`].
///
/// [EIP-2718]: https://eips.ethereum.org/EIPS/eip-2718
pub trait Eip2718Envelope: Decodable2718 + Encodable2718 {}
impl<T> Eip2718Envelope for T where T: Decodable2718 + Encodable2718 {}
