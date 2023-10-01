use alloy_json_rpc::{Id, Request, ResponsePayload, RpcParam};
use alloy_primitives::U256;
use serde_json::value::RawValue;
use tokio::sync::oneshot;

use crate::TransportError;

/// An in-flight JSON-RPC request.
///
/// This struct contains the request that was sent, as well as a channel to
/// receive the response on.
pub struct InFlight {
    pub id: Id,

    /// The request
    pub request: Request<Box<RawValue>>,

    /// The channel to send the response on.
    pub tx: oneshot::Sender<Result<ResponsePayload, TransportError>>,
}

impl std::fmt::Debug for InFlight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let channel_desc = format!(
            "Channel status: {}",
            if self.tx.is_closed() { "closed" } else { "ok" }
        );

        f.debug_struct("InFlight")
            .field("req", &self.request)
            .field("tx", &channel_desc)
            .finish()
    }
}

impl InFlight {
    /// Create a new in-flight request.
    pub fn new<T>(
        request: Request<T>,
    ) -> (
        Self,
        oneshot::Receiver<Result<ResponsePayload, TransportError>>,
    )
    where
        T: RpcParam,
    {
        let id = request.id.clone();

        let request = request.box_params();
        let (tx, rx) = oneshot::channel();

        (Self { id, request, tx }, rx)
    }

    /// Get the method
    pub fn method(&self) -> &'static str {
        self.request.method
    }

    /// Serialize the request as a boxed [`RawValue`].
    ///
    /// This is used to (re-)send the request over the transport.
    pub fn req_json(&self) -> serde_json::Result<Box<RawValue>> {
        serde_json::to_string(&self.request).and_then(RawValue::from_string)
    }

    /// Fulfill the request with a response. This consumes the in-flight
    /// request. If the request is a subscription and the response is not an
    /// error, the subscription ID and the in-flight request are returned.
    pub fn fulfill(self, resp: ResponsePayload) -> Option<(U256, Self)> {
        if self.method() == "eth_subscribe" {
            if let ResponsePayload::Success(val) = resp {
                let sub_id: serde_json::Result<U256> = serde_json::from_str(val.get());
                match sub_id {
                    Ok(alias) => return Some((alias, self)),
                    Err(e) => {
                        let _ = self.tx.send(Err(TransportError::deser_err(e, val.get())));
                        return None;
                    }
                }
            }
        }

        let _ = self.tx.send(Ok(resp));
        None
    }
}
