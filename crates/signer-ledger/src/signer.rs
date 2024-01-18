//! Ledger Ethereum app wrapper.

use crate::types::{DerivationType, LedgerError, INS, P1, P1_FIRST, P2};
use alloy_primitives::{hex, Address, ChainId, B256};
use alloy_signer::{Result, SignableTx, Signature, Signer, TransactionExt};
use async_trait::async_trait;
use coins_ledger::{
    common::{APDUCommand, APDUData},
    transports::{Ledger, LedgerAsync},
};
use futures_util::lock::Mutex;

#[cfg(feature = "eip712")]
use alloy_sol_types::{Eip712Domain, SolStruct};

/// A Ledger Ethereum signer.
///
/// This is a simple wrapper around the [Ledger transport](Ledger).
///
/// Note that this signer only supports asynchronous operations. Calling a non-asynchronous method
/// will always return an error.
#[derive(Debug)]
pub struct LedgerSigner {
    transport: Mutex<Ledger>,
    derivation: DerivationType,
    pub(crate) chain_id: Option<ChainId>,
    pub(crate) address: Address,
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Signer for LedgerSigner {
    async fn sign_hash(&self, _hash: B256) -> Result<Signature> {
        Err(alloy_signer::Error::UnsupportedOperation(
            alloy_signer::UnsupportedSignerOperation::SignHash,
        ))
    }

    #[inline]
    async fn sign_message(&self, message: &[u8]) -> Result<Signature> {
        let mut payload = Self::path_to_bytes(&self.derivation);
        payload.extend_from_slice(&(message.len() as u32).to_be_bytes());
        payload.extend_from_slice(message);

        self.sign_payload(INS::SIGN_PERSONAL_MESSAGE, &payload)
            .await
            .map_err(alloy_signer::Error::other)
    }

    #[inline]
    async fn sign_transaction(&self, tx: &mut SignableTx) -> Result<Signature> {
        let chain_id = self.chain_id();
        if let Some(chain_id) = chain_id {
            tx.set_chain_id_checked(chain_id)?;
        }
        let rlp = tx.rlp_encode();
        let mut sig = self.sign_tx_rlp(&rlp).await.map_err(alloy_signer::Error::other)?;
        if let Some(chain_id) = chain_id.or_else(|| tx.chain_id()) {
            sig = sig.with_chain_id(chain_id);
        }
        Ok(sig)
    }

    #[cfg(feature = "eip712")]
    #[inline]
    async fn sign_typed_data<T: SolStruct + Send + Sync>(
        &self,
        payload: &T,
        domain: &Eip712Domain,
    ) -> Result<Signature> {
        self.sign_typed_data_(payload, domain).await.map_err(alloy_signer::Error::other)
    }

    #[inline]
    fn address(&self) -> Address {
        self.address
    }

    #[inline]
    fn chain_id(&self) -> Option<ChainId> {
        self.chain_id
    }

    #[inline]
    fn set_chain_id(&mut self, chain_id: Option<ChainId>) {
        self.chain_id = chain_id;
    }
}

impl LedgerSigner {
    /// Instantiate the application by acquiring a lock on the ledger device.
    ///
    /// # Examples
    ///
    /// ```
    /// # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
    /// use alloy_signer_ledger::{HDPath, Ledger};
    ///
    /// let ledger = Ledger::new(HDPath::LedgerLive(0), Some(1)).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(
        derivation: DerivationType,
        chain_id: Option<ChainId>,
    ) -> Result<Self, LedgerError> {
        let transport = Ledger::init().await?;
        let address = Self::get_address_with_path_transport(&transport, &derivation).await?;

        Ok(Self { transport: Mutex::new(transport), derivation, chain_id, address })
    }

    /// Get the account which corresponds to our derivation path
    pub async fn get_address(&self) -> Result<Address, LedgerError> {
        self.get_address_with_path(&self.derivation).await
    }

    /// Gets the account which corresponds to the provided derivation path
    pub async fn get_address_with_path(
        &self,
        derivation: &DerivationType,
    ) -> Result<Address, LedgerError> {
        let transport = self.transport.lock().await;
        Self::get_address_with_path_transport(&transport, derivation).await
    }

    #[instrument(skip(transport))]
    async fn get_address_with_path_transport(
        transport: &Ledger,
        derivation: &DerivationType,
    ) -> Result<Address, LedgerError> {
        let data = APDUData::new(&Self::path_to_bytes(derivation));

        let command = APDUCommand {
            ins: INS::GET_PUBLIC_KEY as u8,
            p1: P1::NON_CONFIRM as u8,
            p2: P2::NO_CHAINCODE as u8,
            data,
            response_len: None,
        };

        debug!("Dispatching get_address request to ethereum app");
        let answer = transport.exchange(&command).await?;
        let result = answer.data().ok_or(LedgerError::UnexpectedNullResponse)?;

        let address = {
            // extract the address from the response
            let offset = 1 + result[0] as usize;
            let address_str = &result[offset + 1..offset + 1 + result[offset] as usize];
            let mut address = [0; 20];
            address.copy_from_slice(&hex::decode(address_str)?);
            Address::from(address)
        };
        debug!(?address, "Received address from device");
        Ok(address)
    }

    /// Returns the semver of the Ethereum ledger app
    pub async fn version(&self) -> Result<semver::Version, LedgerError> {
        let transport = self.transport.lock().await;

        let command = APDUCommand {
            ins: INS::GET_APP_CONFIGURATION as u8,
            p1: P1::NON_CONFIRM as u8,
            p2: P2::NO_CHAINCODE as u8,
            data: APDUData::new(&[]),
            response_len: None,
        };

        debug!("Dispatching get_version");
        let answer = transport.exchange(&command).await?;
        let data = answer.data().ok_or(LedgerError::UnexpectedNullResponse)?;
        let &[_flags, major, minor, patch] = data else {
            return Err(LedgerError::ShortResponse { got: data.len(), expected: 4 });
        };
        let version = semver::Version::new(major as u64, minor as u64, patch as u64);
        debug!(%version, "Retrieved version from device");
        Ok(version)
    }

    /// Signs an Ethereum transaction's RLP bytes (requires confirmation on the ledger).
    ///
    /// Note that this does not apply EIP-155.
    pub async fn sign_tx_rlp(&self, tx_rlp: &[u8]) -> Result<Signature, LedgerError> {
        let mut payload = Self::path_to_bytes(&self.derivation);
        payload.extend_from_slice(tx_rlp);
        self.sign_payload(INS::SIGN, &payload).await
    }

    #[cfg(feature = "eip712")]
    async fn sign_typed_data_<T: SolStruct>(
        &self,
        payload: &T,
        domain: &Eip712Domain,
    ) -> Result<Signature, LedgerError> {
        // See comment for v1.6.0 requirement
        // https://github.com/LedgerHQ/app-ethereum/issues/105#issuecomment-765316999
        const EIP712_MIN_VERSION: &str = ">=1.6.0";
        let req = semver::VersionReq::parse(EIP712_MIN_VERSION).unwrap();
        let version = self.version().await?;

        // Enforce app version is greater than EIP712_MIN_VERSION
        if !req.matches(&version) {
            return Err(LedgerError::UnsupportedAppVersion(EIP712_MIN_VERSION));
        }

        let mut data = Self::path_to_bytes(&self.derivation);
        data.extend_from_slice(domain.separator().as_slice());
        data.extend_from_slice(payload.eip712_hash_struct().as_slice());

        self.sign_payload(INS::SIGN_ETH_EIP_712, &data).await
    }

    /// Helper function for signing either transaction data, personal messages or EIP712 derived
    /// structs.
    #[instrument(err, skip_all, fields(command = %command, payload = hex::encode(payload)))]
    async fn sign_payload(&self, command: INS, payload: &[u8]) -> Result<Signature, LedgerError> {
        let transport = self.transport.lock().await;
        let mut command = APDUCommand {
            ins: command as u8,
            p1: P1_FIRST,
            p2: P2::NO_CHAINCODE as u8,
            data: APDUData::new(&[]),
            response_len: None,
        };

        let mut answer = None;
        // workaround for https://github.com/LedgerHQ/app-ethereum/issues/409
        // TODO: remove in future version
        let chunk_size =
            (0..=255).rev().find(|i| payload.len() % i != 3).expect("true for any length");

        // Iterate in 255 byte chunks
        for chunk in payload.chunks(chunk_size) {
            command.data = APDUData::new(chunk);

            debug!("Dispatching packet to device");

            let ans = transport.exchange(&command).await?;
            let data = ans.data().ok_or(LedgerError::UnexpectedNullResponse)?;
            debug!(response = hex::encode(data), "Received response from device");
            answer = Some(ans);

            // We need more data
            command.p1 = P1::MORE as u8;
        }
        drop(transport);

        let answer = answer.unwrap();
        let data = answer.data().unwrap();
        if data.len() != 65 {
            return Err(LedgerError::ShortResponse { got: data.len(), expected: 65 });
        }

        let sig = Signature::from_bytes_and_parity(&data[1..], data[0] as u64)?;
        debug!(?sig, "Received signature from device");
        Ok(sig)
    }

    // helper which converts a derivation path to bytes
    fn path_to_bytes(derivation: &DerivationType) -> Vec<u8> {
        let derivation = derivation.to_string();
        let elements = derivation.split('/').skip(1).collect::<Vec<_>>();
        let depth = elements.len();

        let mut bytes = vec![depth as u8];
        for derivation_index in elements {
            let hardened = derivation_index.contains('\'');
            let mut index = derivation_index.replace('\'', "").parse::<u32>().unwrap();
            if hardened {
                index |= 0x80000000;
            }

            bytes.extend(index.to_be_bytes());
        }

        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{address, U256};

    const DTYPE: DerivationType = DerivationType::LedgerLive(0);

    fn my_address() -> Address {
        std::env::var("LEDGER_ADDRESS").unwrap().parse().unwrap()
    }

    async fn init_ledger() -> LedgerSigner {
        match LedgerSigner::new(DTYPE, Some(1)).await {
            Ok(ledger) => ledger,
            Err(e) => panic!("{e:?}\n{e}"),
        }
    }

    #[tokio::test]
    #[serial_test::serial]
    #[ignore]
    async fn test_get_address() {
        let ledger = init_ledger().await;
        assert_eq!(ledger.get_address().await.unwrap(), my_address());
        assert_eq!(ledger.get_address_with_path(&DTYPE).await.unwrap(), my_address(),);
    }

    #[tokio::test]
    #[serial_test::serial]
    #[ignore]
    async fn test_version() {
        let ledger = init_ledger().await;
        let version = ledger.version().await.unwrap();
        eprintln!("{version}");
        assert!(version.major >= 1);
    }

    #[tokio::test]
    #[serial_test::serial]
    #[ignore]
    async fn test_sign_tx() {
        let ledger = init_ledger().await;

        // approve uni v2 router 0xff
        let data = hex::decode("095ea7b30000000000000000000000007a250d5630b4cf539739df2c5dacb4c659f2488dffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap();

        let mut tx = alloy_consensus::TxLegacy {
            nonce: 0,
            gas_price: 400e9 as u128,
            gas_limit: 1000000,
            to: alloy_consensus::TxKind::Call(address!("2ed7afa17473e17ac59908f088b4371d28585476")),
            input: data.into(),
            value: U256::from(100e18 as u128),
            chain_id: None,
        };
        let sighash = tx.signature_hash();
        let sig = ledger.sign_transaction(&mut tx).await.unwrap();
        assert_eq!(tx.chain_id, None);
        assert_eq!(sig.recover_address_from_prehash(sighash).unwrap(), my_address());
    }

    #[tokio::test]
    #[serial_test::serial]
    #[ignore]
    async fn test_sign_message() {
        let ledger = init_ledger().await;
        let message = "hello world";
        let sig = ledger.sign_message(message.as_bytes()).await.unwrap();
        let addr = ledger.get_address().await.unwrap();
        assert_eq!(addr, my_address());
        assert_eq!(sig.recover_address_from_msg(message.as_bytes()).unwrap(), my_address());
    }
}
