mod builder;
pub use builder::EthereumTxBuilder;

mod eip1559;
pub use eip1559::TxEip1559;

mod eip2930;
pub use eip2930::TxEip2930;

mod eip4844;
pub use eip4844::TxEip4844;

mod envelope;
pub use envelope::{TxEnvelope, TxType};

mod legacy;
pub use legacy::TxLegacy;

mod typed;
pub use typed::TypedTransaction;

/*
use alloy_network::{NetworkSigner, SignableTransaction, TxSigner};
use alloy_primitives::Signature;

use async_trait::async_trait;

use crate::Ethereum;


todo: this doesn't work

error[E0210]: type parameter `S` must be covered by another type when it appears before the first local type (`Ethereum`)
  --> crates/consensus/src/transaction/mod.rs:32:6
   |
32 | impl<S> NetworkSigner<Ethereum> for S
   |      ^ type parameter `S` must be covered by another type when it appears before the first local type (`Ethereum`)
   |
   = note: implementing a foreign trait is only possible if at least one of the types for which it is implemented is local, and no uncovered type parameters appear before that first local type
   = note: in this case, 'before' refers to the following order: `impl<..> ForeignTrait<T1, ..., Tn> for T0`, where `T0` is the first and `Tn` is the last

a solution would be to move this to alloy-network, but then we'd have a circular dep, since alloy-consensus depends on alloy-network for the tx builder traits.
if we move e.g. `Ethereum` to alloy-network, we'd still have a circular dep. the only way to not have a circular dep is to move more
stuff from alloy-network into alloy-consensus or vice versa

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<S> NetworkSigner<Ethereum> for S
where
    S: TxSigner<Signature>,
{
    async fn sign(&self, tx: TypedTransaction) -> alloy_signer::Result<TxEnvelope> {
        match tx {
            TypedTransaction::Legacy(t) => {
                let sig = self.signer.sign_transaction(&t).await?;
                Ok(t.into_signed(sig).into())
            }
            TypedTransaction::Eip2930(t) => {
                let sig = self.signer.sign_transaction(&t).await?;
                Ok(t.into_signed(sig).into())
            }
            TypedTransaction::Eip1559(t) => {
                let sig = self.signer.sign_transaction(&t).await?;
                Ok(t.into_signed(sig).into())
            }
            TypedTransaction::Eip4844(t) => {
                let sig = self.signer.sign_transaction(&t).await?;
                Ok(t.into_signed(sig).into())
            }
        }
    }
}

#[cfg(test)]
mod test {
    use alloy_network::{TxSigner, TxSignerSync};
    use alloy_primitives::{address, ChainId, Signature, U256};
    use alloy_signer::{Result, Signer};

    use crate::TxLegacy;

    #[tokio::test]
    async fn signs_tx() {
        async fn sign_tx_test(tx: &mut TxLegacy, chain_id: Option<ChainId>) -> Result<Signature> {
            let mut before = tx.clone();
            let sig = sign_dyn_tx_test(tx, chain_id).await?;
            if let Some(chain_id) = chain_id {
                assert_eq!(tx.chain_id, Some(chain_id), "chain ID was not set");
                before.chain_id = Some(chain_id);
            }
            assert_eq!(*tx, before);
            Ok(sig)
        }

        async fn sign_dyn_tx_test(
            tx: &mut dyn alloy_network::SignableTransaction,
            chain_id: Option<ChainId>,
        ) -> Result<Signature> {
            let mut wallet: alloy_signer::Wallet<k256::ecdsa::SigningKey> =
                "4c0883a69102937d6231471b5dbb6204fe5129617082792ae468d01a3f362318".parse().unwrap();
            wallet.set_chain_id(chain_id);

            let sig = wallet.sign_transaction_sync(tx)?;
            let sighash = tx.signature_hash();
            assert_eq!(sig.recover_address_from_prehash(&sighash).unwrap(), wallet.address());

            let sig_async = wallet.sign_transaction(tx).await.unwrap();
            assert_eq!(sig_async, sig);

            Ok(sig)
        }

        // retrieved test vector from:
        // https://web3js.readthedocs.io/en/v1.2.0/web3-eth-accounts.html#eth-accounts-signtransaction
        let mut tx = TxLegacy {
            to: crate::TxKind::Call(address!("F0109fC8DF283027b6285cc889F5aA624EaC1F55")),
            value: U256::from(1_000_000_000),
            gas_limit: 2_000_000,
            nonce: 0,
            gas_price: 21_000_000_000,
            input: Default::default(),
            chain_id: None,
        };
        let sig_none = sign_tx_test(&mut tx, None).await.unwrap();

        tx.chain_id = Some(1);
        let sig_1 = sign_tx_test(&mut tx, None).await.unwrap();
        let expected = "c9cf86333bcb065d140032ecaab5d9281bde80f21b9687b3e94161de42d51895727a108a0b8d101465414033c3f705a9c7b826e596766046ee1183dbc8aeaa6825".parse().unwrap();
        assert_eq!(sig_1, expected);
        assert_ne!(sig_1, sig_none);

        tx.chain_id = Some(2);
        let sig_2 = sign_tx_test(&mut tx, None).await.unwrap();
        assert_ne!(sig_2, sig_1);
        assert_ne!(sig_2, sig_none);

        // Sets chain ID.
        tx.chain_id = None;
        let sig_none_none = sign_tx_test(&mut tx, None).await.unwrap();
        assert_eq!(sig_none_none, sig_none);

        tx.chain_id = None;
        let sig_none_1 = sign_tx_test(&mut tx, Some(1)).await.unwrap();
        assert_eq!(sig_none_1, sig_1);

        tx.chain_id = None;
        let sig_none_2 = sign_tx_test(&mut tx, Some(2)).await.unwrap();
        assert_eq!(sig_none_2, sig_2);

        // Errors on mismatch.
        tx.chain_id = Some(2);
        let error = sign_tx_test(&mut tx, Some(1)).await.unwrap_err();
        let expected_error = alloy_signer::Error::TransactionChainIdMismatch { signer: 1, tx: 2 };
        assert_eq!(error.to_string(), expected_error.to_string());
    }
}
*/
