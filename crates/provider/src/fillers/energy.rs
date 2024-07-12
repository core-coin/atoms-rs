use crate::{
    fillers::{FillerControlFlow, TxFiller},
    provider::SendableTx,
    utils::Eip1559Estimation,
    Provider,
};
use alloy_json_rpc::RpcError;
use alloy_network::{Network, TransactionBuilder};
use alloy_rpc_types::BlockNumberOrTag;
use alloy_transport::{Transport, TransportResult};
use futures::FutureExt;

/// An enum over the different types of energy fillable.
#[allow(unreachable_pub)]
#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnergyFillable {
    Legacy { energy_limit: u128, energy_price: u128 },
    Eip1559 { energy_limit: u128, estimate: Eip1559Estimation },
    Eip4844 { energy_limit: u128, estimate: Eip1559Estimation, max_fee_per_blob_energy: u128 },
}

/// A [`TxFiller`] that populates energy related fields in transaction requests if
/// unset.
///
/// Energy related fields are energy_price, energy_limit, max_fee_per_energy
/// max_priority_fee_per_energy and max_fee_per_blob_energy.
///
/// The layer fetches the estimations for these via the
/// [`Provider::get_energy_price`], [`Provider::estimate_energy`] and
/// [`Provider::estimate_eip1559_fees`] methods.
///
/// ## Note:
///
/// The layer will populate energy fields based on the following logic:
/// - if `energy_price` is set, it will process as a legacy tx and populate the
///  `energy_limit` field if unset.
/// - if `access_list` is set, it will process as a 2930 tx and populate the
///  `energy_limit` and `energy_price` field if unset.
/// - if `blob_sidecar` is set, it will process as a 4844 tx and populate the
///  `energy_limit`, `max_fee_per_energy`, `max_priority_fee_per_energy` and
///  `max_fee_per_blob_energy` fields if unset.
/// - Otherwise, it will process as a EIP-1559 tx and populate the `energy_limit`,
///  `max_fee_per_energy` and `max_priority_fee_per_energy` fields if unset.
/// - If the network does not support EIP-1559, it will fallback to the legacy
///  tx and populate the `energy_limit` and `energy_price` fields if unset.
///
/// # Example
///
/// ```
/// # use alloy_network::{NetworkSigner, EthereumSigner, Ethereum};
/// # use alloy_rpc_types::TransactionRequest;
/// # use alloy_provider::{ProviderBuilder, RootProvider, Provider};
/// # async fn test<S: NetworkSigner<Ethereum> + Clone>(url: url::Url, signer: S) -> Result<(), Box<dyn std::error::Error>> {
/// let provider = ProviderBuilder::new()
///     .with_energy_estimation()
///     .signer(signer)
///     .on_http(url);
///
/// provider.send_transaction(TransactionRequest::default()).await;
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct EnergyFiller;

impl EnergyFiller {
    async fn prepare_legacy<P, T, N>(
        &self,
        provider: &P,
        tx: &N::TransactionRequest,
    ) -> TransportResult<EnergyFillable>
    where
        P: Provider<T, N>,
        T: Transport + Clone,
        N: Network,
    {
        let energy_price_fut = if let Some(energy_price) = tx.energy_price() {
            async move { Ok(energy_price) }.left_future()
        } else {
            async { provider.get_energy_price().await }.right_future()
        };

        let energy_limit_fut = if let Some(energy_limit) = tx.energy_limit() {
            async move { Ok(energy_limit) }.left_future()
        } else {
            async { provider.estimate_energy(tx, Default::default()).await }.right_future()
        };

        let (energy_price, energy_limit) = futures::try_join!(energy_price_fut, energy_limit_fut)?;

        Ok(EnergyFillable::Legacy { energy_limit, energy_price })
    }

    // async fn prepare_1559<P, T, N>(
    //     &self,
    //     provider: &P,
    //     tx: &N::TransactionRequest,
    // ) -> TransportResult<EnergyFillable>
    // where
    //     P: Provider<T, N>,
    //     T: Transport + Clone,
    //     N: Network,
    // {
    //     let energy_limit_fut = if let Some(energy_limit) = tx.energy_limit() {
    //         async move { Ok(energy_limit) }.left_future()
    //     } else {
    //         async { provider.estimate_energy(tx, Default::default()).await }.right_future()
    //     };

    //     let eip1559_fees_fut = if let (
    //         Some(max_fee_per_energy),
    //         Some(max_priority_fee_per_energy),
    //     ) = (tx.max_fee_per_gas(), tx.max_priority_fee_per_gas())
    //     {
    //         async move { Ok(Eip1559Estimation { max_fee_per_energy, max_priority_fee_per_energy }) }
    //             .left_future()
    //     } else {
    //         async { provider.estimate_eip1559_fees(None).await }.right_future()
    //     };

    //     let (energy_limit, estimate) = futures::try_join!(energy_limit_fut, eip1559_fees_fut)?;

    //     Ok(EnergyFillable::Eip1559 { energy_limit, estimate })
    // }

    //     async fn prepare_4844<P, T, N>(
    //         &self,
    //         provider: &P,
    //         tx: &N::TransactionRequest,
    //     ) -> TransportResult<EnergyFillable>
    //     where
    //         P: Provider<T, N>,
    //         T: Transport + Clone,
    //         N: Network,
    //     {
    //         let energy_limit_fut = if let Some(energy_limit) = tx.energy_limit() {
    //             async move { Ok(energy_limit) }.left_future()
    //         } else {
    //             async { provider.estimate_energy(tx, Default::default()).await }.right_future()
    //         };

    //         let eip1559_fees_fut = if let (
    //             Some(max_fee_per_energy),
    //             Some(max_priority_fee_per_energy),
    //         ) = (tx.max_fee_per_gas(), tx.max_priority_fee_per_gas())
    //         {
    //             async move { Ok(Eip1559Estimation { max_fee_per_energy, max_priority_fee_per_energy }) }
    //                 .left_future()
    //         } else {
    //             async { provider.estimate_eip1559_fees(None).await }.right_future()
    //         };

    //         let max_fee_per_blob_energy_fut =
    //             if let Some(max_fee_per_blob_energy) = tx.max_fee_per_gas() {
    //                 async move { Ok(max_fee_per_blob_energy) }.left_future()
    //             } else {
    //                 async {
    //                     provider
    //                         .get_block_by_number(BlockNumberOrTag::Latest, false)
    //                         .await?
    //                         .ok_or(RpcError::NullResp)?
    //                         .header
    //                         .next_block_blob_fee()
    //                         .ok_or(RpcError::UnsupportedFeature("eip4844"))
    //                 }
    //                 .right_future()
    //             };

    //         let (energy_limit, estimate, max_fee_per_blob_energy) =
    //             futures::try_join!(energy_limit_fut, eip1559_fees_fut, max_fee_per_blob_energy_fut)?;

    //         Ok(EnergyFillable::Eip4844 { energy_limit, estimate, max_fee_per_blob_energy })
    //     }
}

impl<N: Network> TxFiller<N> for EnergyFiller {
    type Fillable = EnergyFillable;

    fn status(&self, tx: &<N as Network>::TransactionRequest) -> FillerControlFlow {
        // legacy and eip2930 tx
        if tx.energy_price().is_some() && tx.energy_limit().is_some() {
            return FillerControlFlow::Finished;
        }

        // 4844
        // if tx.max_fee_per_blob_gas().is_some()
        //     && tx.max_fee_per_gas().is_some()
        //     && tx.max_priority_fee_per_gas().is_some()
        //     && tx.energy_limit().is_some()
        // {
        //     return FillerControlFlow::Finished;
        // }

        // // eip1559
        // if tx.blob_sidecar().is_none()
        //     && tx.max_fee_per_gas().is_some()
        //     && tx.max_priority_fee_per_gas().is_some()
        //     && tx.energy_limit().is_some()
        // {
        //     return FillerControlFlow::Finished;
        // }

        FillerControlFlow::Ready
    }

    async fn prepare<P, T>(
        &self,
        provider: &P,
        tx: &<N as Network>::TransactionRequest,
    ) -> TransportResult<Self::Fillable>
    where
        P: Provider<T, N>,
        T: Transport + Clone,
    {
        self.prepare_legacy(provider, tx).await
    }

    async fn fill(
        &self,
        fillable: Self::Fillable,
        mut tx: SendableTx<N>,
    ) -> TransportResult<SendableTx<N>> {
        if let Some(builder) = tx.as_mut_builder() {
            match fillable {
                EnergyFillable::Legacy {
                    energy_limit: energy_limit,
                    energy_price: energy_price,
                } => {
                    builder.set_energy_limit(energy_limit);
                    builder.set_energy_price(energy_price);
                }
                EnergyFillable::Eip1559 { energy_limit: energy_limit, estimate } => {
                    builder.set_energy_limit(energy_limit);
                    builder.set_max_fee_per_gas(estimate.max_fee_per_energy);
                    builder.set_max_priority_fee_per_gas(estimate.max_priority_fee_per_energy);
                }
                EnergyFillable::Eip4844 {
                    energy_limit: energy_limit,
                    estimate,
                    max_fee_per_blob_energy: max_fee_per_blob_energy,
                } => {
                    builder.set_energy_limit(energy_limit);
                    builder.set_max_fee_per_gas(estimate.max_fee_per_energy);
                    builder.set_max_priority_fee_per_gas(estimate.max_priority_fee_per_energy);
                    builder.set_max_fee_per_blob_gas(max_fee_per_blob_energy);
                }
            }
        };
        Ok(tx)
    }
}

#[cfg(feature = "reqwest")]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ProviderBuilder, WalletProvider};
    use base_primitives::{cAddress, U256};
    use alloy_rpc_types::TransactionRequest;

    #[tokio::test]
    async fn no_energy_price_or_limit() {
        let provider = ProviderBuilder::new().with_recommended_fillers().on_anvil_with_signer();
        let from = provider.default_signer_address();
        // EnergyEstimationLayer requires chain_id to be set to handle EIP-1559 tx
        let tx = TransactionRequest {
            from: Some(from),
            value: Some(U256::from(100)),
            to: Some(cAddress!("0000d8dA6BF26964aF9D7eEd9e03E53415D37aA96045").into()),
            network_id: 1,
            ..Default::default()
        };

        let tx = provider.send_transaction(tx).await.unwrap();

        let tx = tx.get_receipt().await.unwrap();

        // assert_eq!(tx.effective_gas_price, 0x3b9aca00);
        assert_eq!(tx.energy_used, 0x5208);
    }

    #[tokio::test]
    async fn no_energy_limit() {
        let provider = ProviderBuilder::new().with_recommended_fillers().on_anvil_with_signer();

        let from = provider.default_signer_address();

        let energy_price = provider.get_energy_price().await.unwrap();
        let tx = TransactionRequest {
            from: Some(from),
            value: Some(U256::from(100)),
            to: Some(cAddress!("0000d8dA6BF26964aF9D7eEd9e03E53415D37aA96045").into()),
            energy_price: Some(energy_price),
            ..Default::default()
        };

        let tx = provider.send_transaction(tx).await.unwrap();

        let receipt = tx.get_receipt().await.unwrap();

        assert_eq!(receipt.energy_used, 0x5208);
    }
}
