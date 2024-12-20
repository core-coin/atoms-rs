use crate::{SignableTransaction, Signed, Transaction};
use alloy_rlp::{length_of_length, BufMut, Decodable, Encodable, Header, Result};
use base_primitives::{sha3, Bytes, ChainId, Signature, TxKind, U256};
use core::mem;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

/// Legacy transaction.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct TxLegacy {
    /// Added as EIP-155: Simple replay attack protection
    #[cfg_attr(feature = "serde", serde(default, with = "atoms_serde::u64_via_ruint",))]
    pub network_id: ChainId,
    /// A scalar value equal to the number of transactions sent by the sender; formally Tn.
    #[cfg_attr(feature = "serde", serde(with = "atoms_serde::u64_via_ruint"))]
    pub nonce: u64,
    /// A scalar value equal to the number of
    /// Ore to be paid per unit of energy for all computation
    /// costs incurred as a result of the execution of this transaction; formally Tp.
    ///
    /// As core circulation is around 120mil xcb as of 2022 that is around
    /// 120000000000000000000000000 ore we are safe to use u128 as its max number is:
    /// 340282366920938463463374607431768211455
    #[cfg_attr(feature = "serde", serde(with = "atoms_serde::u128_via_ruint"))]
    pub energy_price: u128,
    /// A scalar value equal to the maximum
    /// amount of energy that should be used in executing
    /// this transaction. This is paid up-front, before any
    /// computation is done and may not be increased
    /// later; formally Tg.
    #[cfg_attr(feature = "serde", serde(with = "atoms_serde::u128_via_ruint"))]
    pub energy_limit: u128,
    /// The 160-bit address of the message call’s recipient or, for a contract creation
    /// transaction, ∅, used here to denote the only member of B0 ; formally Tt.
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "TxKind::is_create"))]
    pub to: TxKind,
    /// A scalar value equal to the number of Wei to
    /// be transferred to the message call’s recipient or,
    /// in the case of contract creation, as an endowment
    /// to the newly created account; formally Tv.
    pub value: U256,
    /// Input has two uses depending if transaction is Create or Call (if `to` field is None or
    /// Some). pub init: An unlimited size byte array specifying the
    /// EVM-code for the account initialisation procedure CREATE,
    /// data: An unlimited size byte array specifying the
    /// input data of the message call, formally Td.
    pub input: Bytes,
}

impl TxLegacy {
    /// The EIP-2718 transaction type.
    pub const TX_TYPE: isize = 0;

    /// Calculates a heuristic for the in-memory size of the [TxLegacy] transaction.
    #[inline]
    pub fn size(&self) -> usize {
        mem::size_of::<Option<ChainId>>() + // chain_id
        mem::size_of::<u64>() + // nonce
        mem::size_of::<u128>() + // gas_price
        mem::size_of::<u64>() + // gas_limit
        self.to.size() + // to
        mem::size_of::<U256>() + // value
        self.input.len() // input
    }

    /// Outputs the length of the transaction's fields, without a RLP header or length of the
    /// eip155 fields.
    #[doc(hidden)]
    pub fn fields_len(&self) -> usize {
        let mut len = 0;
        len += self.nonce.length();
        len += self.energy_price.length();
        len += self.energy_limit.length();
        len += self.to.length();
        len += self.value.length();
        len += self.input.0.length();
        len += self.chain_id().length();
        len
    }

    /// Encodes only the transaction's fields into the desired buffer, without a RLP header or
    /// eip155 fields.
    pub(crate) fn encode_fields(&self, out: &mut dyn BufMut) {
        self.nonce.encode(out);
        self.energy_price.encode(out);
        self.energy_limit.encode(out);
        self.chain_id().encode(out);
        self.to.encode(out);
        self.value.encode(out);
        self.input.0.encode(out);
    }

    /// Encodes the transaction from RLP bytes, including the signature. This __does not__ encode a
    /// tx type byte or string header.
    ///
    /// This __does__ encode a list header and include a signature.
    pub fn encode_with_signature_fields(
        &self,
        signature: &Signature,
        out: &mut dyn alloy_rlp::BufMut,
    ) {
        let payload_length = self.fields_len() + signature.rlp_len();
        let header = Header { list: true, payload_length };
        header.encode(out);
        self.encode_fields(out);
        signature.write_rlp(out);
    }

    /// Returns what the encoded length should be, if the transaction were RLP encoded with the
    /// given signature.
    pub(crate) fn encoded_len_with_signature(&self, signature: &Signature) -> usize {
        let payload_length = self.fields_len() + signature.rlp_len();
        Header { list: true, payload_length }.length() + payload_length
    }

    /// Encodes EIP-155 arguments into the desired buffer. Only encodes values
    /// for legacy transactions.
    pub(crate) fn encode_eip155_signing_fields(&self, out: &mut dyn BufMut) {
        self.nonce.encode(out);
        self.energy_price.encode(out);
        self.energy_limit.encode(out);
        self.to.encode(out);
        self.value.encode(out);
        self.input.0.encode(out);
        self.chain_id().encode(out);
    }

    /// Outputs the length of EIP-155 fields. Only outputs a non-zero value for EIP-155 legacy
    /// transactions.
    pub(crate) fn eip155_fields_len(&self) -> usize {
        // EIP-155 encodes the chain ID and two zeroes, so we add 2 to the length of the chain
        // ID to get the length of all 3 fields
        // len(chain_id) + (0x00) + (0x00)
        self.network_id.length() + 2
    }

    /// Decodes the transaction from RLP bytes, including the signature.
    ///
    /// This __does not__ expect the bytes to start with a transaction type byte or string
    /// header.
    ///
    /// This __does__ expect the bytes to start with a list header and include a signature.
    #[doc(hidden)]
    pub fn decode_signed_fields(buf: &mut &[u8]) -> alloy_rlp::Result<Signed<Self>> {
        let header = Header::decode(buf)?;
        if !header.list {
            return Err(alloy_rlp::Error::UnexpectedString);
        }

        // record original length so we can check encoding
        let original_len = buf.len();

        let mut tx = Self::decode_fields(buf)?;
        let signature = Signature::decode_rlp_sig(buf)?;

        // extract chain id from signature
        tx.network_id = 0;

        let signed = tx.into_signed(signature);
        if buf.len() + header.payload_length != original_len {
            return Err(alloy_rlp::Error::ListLengthMismatch {
                expected: header.payload_length,
                got: original_len - buf.len(),
            });
        }

        Ok(signed)
    }

    /// Decode the RLP fields of the transaction, without decoding an RLP
    /// header.
    pub(crate) fn decode_fields(data: &mut &[u8]) -> Result<Self> {
        Ok(Self {
            nonce: Decodable::decode(data)?,
            energy_price: Decodable::decode(data)?,
            energy_limit: Decodable::decode(data)?,
            to: Decodable::decode(data)?,
            value: Decodable::decode(data)?,
            input: Decodable::decode(data)?,
            network_id: 0,
        })
    }
}

impl Transaction for TxLegacy {
    fn input(&self) -> &[u8] {
        &self.input
    }

    fn to(&self) -> TxKind {
        self.to
    }

    fn value(&self) -> U256 {
        self.value
    }

    fn chain_id(&self) -> ChainId {
        self.network_id
    }

    fn nonce(&self) -> u64 {
        self.nonce
    }

    fn gas_limit(&self) -> u128 {
        self.energy_limit
    }

    fn gas_price(&self) -> Option<u128> {
        Some(self.energy_price)
    }
}

impl SignableTransaction<Signature> for TxLegacy {
    fn set_chain_id(&mut self, chain_id: ChainId) {
        self.network_id = chain_id;
    }

    fn encode_for_signing(&self, out: &mut dyn BufMut) {
        Header { list: true, payload_length: self.fields_len() }.encode(out);
        // self.encode_fields(out);
        self.encode_eip155_signing_fields(out);
    }

    fn payload_len_for_signature(&self) -> usize {
        let payload_length = self.fields_len();
        // 'header length' + 'payload length'
        Header { list: true, payload_length }.length() + payload_length
    }

    fn into_signed(self, signature: Signature) -> Signed<Self> {
        let mut buf = Vec::with_capacity(self.encoded_len_with_signature(&signature));
        self.encode_with_signature_fields(&signature, &mut buf);
        let hash = sha3(&buf);
        Signed::new_unchecked(self, signature, hash)
    }
}

impl Encodable for TxLegacy {
    fn encode(&self, out: &mut dyn BufMut) {
        self.encode_for_signing(out)
    }

    fn length(&self) -> usize {
        let payload_length = self.fields_len() + self.eip155_fields_len();
        // 'header length' + 'payload length'
        length_of_length(payload_length) + payload_length
    }
}

impl Decodable for TxLegacy {
    fn decode(data: &mut &[u8]) -> Result<Self> {
        let header = Header::decode(data)?;
        let remaining_len = data.len();

        let transaction_payload_len = header.payload_length;

        if transaction_payload_len > remaining_len {
            return Err(alloy_rlp::Error::InputTooShort);
        }

        let mut transaction = Self::decode_fields(data)?;

        // If we still have data, it should be an eip-155 encoded chain_id
        if !data.is_empty() {
            transaction.network_id = Decodable::decode(data)?;
            let _: U256 = Decodable::decode(data)?; // r
            let _: U256 = Decodable::decode(data)?; // s
        }

        let decoded = remaining_len - data.len();
        if decoded != transaction_payload_len {
            return Err(alloy_rlp::Error::UnexpectedLength);
        }

        Ok(transaction)
    }
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use crate::{SignableTransaction, TxLegacy};
    use base_primitives::{address, b256, cAddress, hex, Address, IcanAddress, Signature, TxKind, B256, U256};

    #[test]
    fn recover_signer_legacy() {
        let signer: IcanAddress = hex!("0000398137383b3d25c92898c656696e41950e47316b").into();
        let hash: B256 =
            hex!("bb3a336e3f823ec18197f1e13ee875700f08f03e2cab75f0d0b118dabb44cba0").into();

        let tx = TxLegacy {
            network_id: 1,
            nonce: 0x18,
            energy_price: 0xfa56ea00,
            energy_limit: 119902,
            to: TxKind::Call(hex!("000006012c8cf97bead5deae237070f9587f8e7a266d").into()),
            value: U256::from(0x1c6bf526340000u64),
            input:  hex!("f7d8c88300000000000000000000000000000000000000000000000000000000000cee6100000000000000000000000000000000000000000000000000000000000ac3e1").into(),
        };

        let sig = Signature::from_str("").unwrap();

        let signed_tx = tx.into_signed(sig);

        assert_eq!(*signed_tx.hash(), hash, "Expected same hash");
        assert_eq!(signed_tx.recover_signer(1).unwrap(), signer, "Recovering signer should pass.");
    }

    #[test]
    // Test vector from https://github.com/core-coin/atoms-rs/issues/125
    fn decode_legacy_and_recover_signer() {
        let raw_tx = "f9015482078b8505d21dba0083022ef1947a250d5630b4cf539739df2c5dacb4c659f2488d880c46549a521b13d8b8e47ff36ab50000000000000000000000000000000000000000000066ab5a608bd00a23f2fe000000000000000000000000000000000000000000000000000000000000008000000000000000000000000048c04ed5691981c42154c6167398f95e8f38a7ff00000000000000000000000000000000000000000000000000000000632ceac70000000000000000000000000000000000000000000000000000000000000002000000000000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc20000000000000000000000006c6ee5e31d828de241282b9606c8e98ea48526e225a0c9077369501641a92ef7399ff81c21639ed4fd8fc69cb793cfa1dbfab342e10aa0615facb2f1bcf3274a354cfe384a38d0cc008a11c2dd23a69111bc6930ba27a8";

        let tx = TxLegacy::decode_signed_fields(
            &mut base_primitives::hex::decode(raw_tx).unwrap().as_slice(),
        )
        .unwrap();

        let recovered = tx.recover_signer(1).unwrap();
        let expected = cAddress!("0000a12e1462d0ceD572f396F58B6E2D03894cD7C8a4");

        assert_eq!(tx.tx().network_id, 1, "Expected same chain id");
        assert_eq!(expected, recovered, "Expected same signer");
    }
}
