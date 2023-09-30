use alloy_primitives::{keccak256, B256};
use serde_json::value::RawValue;
use tokio::sync::broadcast;

#[derive(Clone)]
/// An active subscription.
pub struct ActiveSubscription {
    /// The serialized subscription request.
    pub request: Box<RawValue>,
    /// Cached hash of the request, used for sorting and equality.
    pub local_id: B256,
    /// The channel via which notifications are broadcast.
    pub tx: broadcast::Sender<Box<RawValue>>,
}

impl PartialEq for ActiveSubscription {
    fn eq(&self, other: &Self) -> bool {
        self.local_id == other.local_id
    }
}

impl Eq for ActiveSubscription {}

impl PartialOrd for ActiveSubscription {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.local_id.partial_cmp(&other.local_id)
    }
}

impl Ord for ActiveSubscription {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.local_id.cmp(&other.local_id)
    }
}

impl std::fmt::Debug for ActiveSubscription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let channel_desc = format!("Channel status: {} subscribers", self.tx.receiver_count());

        f.debug_struct("ActiveSubscription")
            .field("req", &self.request)
            .field("tx", &channel_desc)
            .finish()
    }
}

impl ActiveSubscription {
    /// Create a new active subscription.
    pub fn new(request: Box<RawValue>) -> (Self, broadcast::Receiver<Box<RawValue>>) {
        let (tx, rx) = broadcast::channel(16);
        let local_id = keccak256(request.get());

        (
            Self {
                request,
                local_id,
                tx,
            },
            rx,
        )
    }

    /// Serialize the request as a boxed [`RawValue`].
    ///
    /// This is used to (re-)send the request over the transport.
    pub fn req_json(&self) -> serde_json::Result<Box<RawValue>> {
        serde_json::to_string(&self.request).and_then(RawValue::from_string)
    }

    /// Notify the subscription channel of a new value, if any receiver exists.
    /// If no receiver exists, the notification is dropped.
    pub fn notify(&mut self, notification: Box<RawValue>) {
        if self.tx.receiver_count() > 0 {
            let _ = self.tx.send(notification);
        }
    }
}
