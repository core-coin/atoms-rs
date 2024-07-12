use crate::Error;
use alloy_network::Ethereum;
use alloy_provider::{FilterPollerBuilder, Network, Provider};
use alloy_rpc_types::{Filter, Log};
use alloy_transport::{Transport, TransportResult};
use base_primitives::{Address, IcanAddress, LogData};
use base_ylm_types::YlmEvent;
use futures::Stream;
use futures_util::StreamExt;
use std::{fmt, marker::PhantomData};

/// Helper for managing the event filter before querying or streaming its logs
#[must_use = "event filters do nothing unless you `query`, `watch`, or `stream` them"]
pub struct Event<T, P, E, N = Ethereum> {
    /// The provider to use for querying or streaming logs.
    pub provider: P,
    /// The filter to use for querying or streaming logs.
    pub filter: Filter,
    _phantom: PhantomData<(T, E, N)>,
}

impl<T, P: fmt::Debug, E, N> fmt::Debug for Event<T, P, E, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Event")
            .field("provider", &self.provider)
            .field("filter", &self.filter)
            .field("event_type", &format_args!("{}", std::any::type_name::<E>()))
            .finish()
    }
}

#[doc(hidden)]
impl<'a, T: Transport + Clone, P: Provider<T, N>, E: YlmEvent, N: Network> Event<T, &'a P, E, N> {
    // `sol!` macro constructor, see `#[sol(rpc)]`. Not public API.
    // NOTE: please avoid changing this function due to its use in the `sol!` macro.
    pub fn new_sol(provider: &'a P, address: &IcanAddress) -> Self {
        // keccak256 hash of the event signature needed for the filter to actually filter by event
        // check that the event is not anonymous to include the event signature in the filter
        if E::ANONYMOUS {
            Self::new(provider, Filter::new().address(*address))
        } else {
            Self::new(provider, Filter::new().address(*address).event_signature(E::SIGNATURE_HASH))
        }
    }
}

impl<T: Transport + Clone, P: Provider<T, N>, E: YlmEvent, N: Network> Event<T, P, E, N> {
    /// Creates a new event with the provided provider and filter.
    #[allow(clippy::missing_const_for_fn)]
    pub fn new(provider: P, filter: Filter) -> Self {
        Self { provider, filter, _phantom: PhantomData }
    }

    /// Queries the blockchain for the selected filter and returns a vector of matching event logs.
    pub async fn query(&self) -> Result<Vec<(E, Log)>, Error> {
        let logs = self.query_raw().await?;
        logs.into_iter().map(|log| Ok((decode_log(&log)?, log))).collect()
    }

    /// Queries the blockchain for the selected filter and returns a vector of matching event logs,
    /// without decoding them.
    pub async fn query_raw(&self) -> TransportResult<Vec<Log>> {
        self.provider.get_logs(&self.filter).await
    }

    /// Watches for events that match the filter.
    ///
    /// Returns a stream of decoded events and raw logs.
    #[doc(alias = "stream")]
    #[doc(alias = "stream_with_meta")]
    pub async fn watch(&self) -> TransportResult<EventPoller<T, E>> {
        let poller = self.provider.watch_logs(&self.filter).await?;
        Ok(poller.into())
    }

    /// Subscribes to the stream of events that match the filter.
    ///
    /// Returns a stream of decoded events and raw logs.
    #[cfg(feature = "pubsub")]
    pub async fn subscribe(&self) -> TransportResult<subscription::EventSubscription<E>> {
        let sub = self.provider.subscribe_logs(&self.filter).await?;
        Ok(sub.into())
    }
}

impl<T, P: Clone, E, N> Event<T, &P, E, N> {
    /// Clones the provider and returns a new event with the cloned provider.
    pub fn with_cloned_provider(self) -> Event<T, P, E, N> {
        Event { provider: self.provider.clone(), filter: self.filter, _phantom: PhantomData }
    }
}

/// An event poller.
///
/// Polling configuration is available through the [`poller`](Self::poller) field.
pub struct EventPoller<T, E> {
    /// The inner poller.
    pub poller: FilterPollerBuilder<T, Log>,
    _phantom: PhantomData<E>,
}

impl<T, E> AsRef<FilterPollerBuilder<T, Log>> for EventPoller<T, E> {
    #[inline]
    fn as_ref(&self) -> &FilterPollerBuilder<T, Log> {
        &self.poller
    }
}

impl<T, E> AsMut<FilterPollerBuilder<T, Log>> for EventPoller<T, E> {
    #[inline]
    fn as_mut(&mut self) -> &mut FilterPollerBuilder<T, Log> {
        &mut self.poller
    }
}

impl<T: fmt::Debug, E> fmt::Debug for EventPoller<T, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EventPoller")
            .field("poller", &self.poller)
            .field("event_type", &format_args!("{}", std::any::type_name::<E>()))
            .finish()
    }
}

impl<T, E> From<FilterPollerBuilder<T, Log>> for EventPoller<T, E> {
    fn from(poller: FilterPollerBuilder<T, Log>) -> Self {
        Self { poller, _phantom: PhantomData }
    }
}

impl<T: Transport + Clone, E: YlmEvent> EventPoller<T, E> {
    /// Starts the poller and returns a stream that yields the decoded event and the raw log.
    ///
    /// Note that this stream will not return `None` until the provider is dropped.
    pub fn into_stream(self) -> impl Stream<Item = base_ylm_types::Result<(E, Log)>> + Unpin {
        self.poller
            .into_stream()
            .flat_map(futures_util::stream::iter)
            .map(|log| decode_log(&log).map(|e| (e, log)))
    }
}

fn decode_log<E: YlmEvent>(log: &Log) -> base_ylm_types::Result<E> {
    let log_data: &LogData = log.as_ref();

    E::decode_raw_log(log_data.topics().iter().copied(), &log_data.data, false)
}

#[cfg(feature = "pubsub")]
pub(crate) mod subscription {
    use super::*;
    use alloy_pubsub::Subscription;

    /// An event subscription.
    ///
    /// Underlying subscription is available through the [`sub`](Self::sub) field.
    pub struct EventSubscription<E> {
        /// The inner poller.
        pub sub: Subscription<Log>,
        _phantom: PhantomData<E>,
    }

    impl<E> AsRef<Subscription<Log>> for EventSubscription<E> {
        #[inline]
        fn as_ref(&self) -> &Subscription<Log> {
            &self.sub
        }
    }

    impl<E> AsMut<Subscription<Log>> for EventSubscription<E> {
        #[inline]
        fn as_mut(&mut self) -> &mut Subscription<Log> {
            &mut self.sub
        }
    }

    impl<E> fmt::Debug for EventSubscription<E> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("EventSubscription")
                .field("sub", &self.sub)
                .field("event_type", &format_args!("{}", std::any::type_name::<E>()))
                .finish()
        }
    }

    impl<E> From<Subscription<Log>> for EventSubscription<E> {
        fn from(sub: Subscription<Log>) -> Self {
            Self { sub, _phantom: PhantomData }
        }
    }

    impl<E: YlmEvent> EventSubscription<E> {
        /// Converts the subscription into a stream.
        pub fn into_stream(self) -> impl Stream<Item = base_ylm_types::Result<(E, Log)>> + Unpin {
            self.sub.into_stream().map(|log| decode_log(&log).map(|e| (e, log)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base_primitives::U256;
    use base_ylm_types::sol;

    sol! {
        // ylem v0.8.24; solc a.sol --optimize --bin
        #[sol(rpc, bytecode = "608060405234801561001057600080fd5b50610163806100206000396000f3fe608060405234801561001057600080fd5b50600436106100365760003560e01c80637f1413e81461003b578063d60caeb414610045575b600080fd5b6100436100a5565b005b6100436040805160608082526005908201526468656c6c6f60d81b60808201526001602082015263deadbeef91810191909152602a907f5e8b344e7d6111f58eb3fc28175df3842157fcc06ef7a3f2f8d64a6aaed536ee9060a0016100fe565b6040805160608082526005908201526468656c6c6f60d81b60808201526001602082015263deadbeef91810191909152602a907fc1e295a8d77bc155215f12a5ee1ba220570806df53b3381939d8ca6529bc8ce79060a0015b60405180910390a256fea2646970667358221220aaca752d7eb1a1caa5d9868d2057603bf6a822b84cae623d962c2f2644088edc64736f6c637827302e382e342d646576656c6f702e323032322e382e32322b636f6d6d69742e61303164646338320058")]
        #[allow(dead_code)]
        contract MyContract {
            #[derive(Debug, PartialEq)]
            event MyEvent(uint64 indexed, string, bool, bytes32);

            #[derive(Debug, PartialEq)]
            event WrongEvent(uint64 indexed, string, bool, bytes32);

            function doEmit() external {
                emit MyEvent(42, "hello", true, bytes32(uint256(0xdeadbeef)));
            }

            function doEmitWrongEvent() external {
                emit WrongEvent(42, "hello", true, bytes32(uint256(0xdeadbeef)));
            }
        }
    }

    #[tokio::test]
    async fn event_filters() {
        let _ = tracing_subscriber::fmt::try_init();

        let provider = alloy_provider::ProviderBuilder::new()
            .with_recommended_fillers()
            .on_anvil_with_signer();

        let contract = MyContract::deploy(&provider).await.unwrap();

        let event: Event<_, _, MyContract::MyEvent, _> = Event::new(&provider, Filter::new());
        let all = event.query().await.unwrap();
        assert_eq!(all.len(), 0);

        // Same as above, but generated by `sol!`.
        let event = contract.MyEvent_filter();

        let poller = event.watch().await.unwrap();

        let _receipt =
            contract.doEmit().send().await.unwrap().get_receipt().await.expect("no receipt");

        let expected_event = MyContract::MyEvent {
            _0: 42,
            _1: "hello".to_string(),
            _2: true,
            _3: U256::from(0xdeadbeefu64).into(),
        };

        let mut stream = poller.into_stream();
        let (stream_event, stream_log) = stream.next().await.unwrap().unwrap();
        assert_eq!(MyContract::MyEvent::SIGNATURE_HASH.0, stream_log.topics().first().unwrap().0); // add check that the received event signature is the same as the one we expect
        assert_eq!(stream_event, expected_event);
        assert_eq!(stream_log.inner.address, *contract.address());
        assert_eq!(stream_log.block_number, Some(2));

        // This is not going to return `None`
        // assert!(stream.next().await.is_none());

        let all = event.query().await.unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].0, expected_event);
        assert_eq!(all[0].1, stream_log);

        // send the wrong event and make sure it is NOT picked up by the event filter
        let _wrong_receipt = contract
            .doEmitWrongEvent()
            .send()
            .await
            .unwrap()
            .get_receipt()
            .await
            .expect("no receipt");

        // we sent the wrong event
        // so no events should be returned when querying event.query() (MyEvent)
        let all = event.query().await.unwrap();
        assert_eq!(all.len(), 0);

        #[cfg(feature = "ws")]
        {
            let provider = alloy_provider::ProviderBuilder::default()
                .on_ws(anvil.ws_endpoint())
                .await
                .unwrap();

            let contract = MyContract::new(*contract.address(), provider);
            let event = contract.MyEvent_filter();

            let sub = event.subscribe().await.unwrap();

            contract.doEmit().send().await.unwrap().get_receipt().await.expect("no receipt");

            let mut stream = sub.into_stream();

            let (stream_event, stream_log) = stream.next().await.unwrap().unwrap();
            assert_eq!(
                MyContract::MyEvent::SIGNATURE_HASH.0,
                stream_log.topics().first().unwrap().0
            );
            assert_eq!(stream_event, expected_event);
            assert_eq!(stream_log.address, *contract.address());
            assert_eq!(stream_log.block_number, Some(U256::from(3)));

            // send the request to emit the wrong event
            contract
                .doEmitWrongEvent()
                .send()
                .await
                .unwrap()
                .get_receipt()
                .await
                .expect("no receipt");

            // we sent the wrong event
            // so no events should be returned when querying event.query() (MyEvent)
            let all = event.query().await.unwrap();
            assert_eq!(all.len(), 0);
        }
    }
}
