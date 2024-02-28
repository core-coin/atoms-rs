use crate::Http;
use alloy_json_rpc::{ErrorPayload, RequestPacket, ResponsePacket};
use alloy_transport::{TransportError, TransportErrorKind, TransportFut};
use std::task;
use tower::Service;

impl Http<reqwest::Client> {
    /// Make a request.
    fn request(&self, req: RequestPacket) -> TransportFut<'static> {
        let this = self.clone();
        Box::pin(async move {
            let resp = this
                .client
                .post(this.url)
                .json(&req)
                .send()
                .await
                .map_err(TransportErrorKind::custom)?;
            let body = resp.bytes().await.map_err(TransportErrorKind::custom)?;

            serde_json::from_slice(&body).map_err(|err| {
                // check if the response is an error payload
                if let Ok(err) = serde_json::from_slice::<ErrorPayload>(&body) {
                    return TransportError::err_resp(err);
                }

                TransportError::deser_err(err, String::from_utf8_lossy(&body))
            })
        })
    }
}

impl Service<RequestPacket> for Http<reqwest::Client> {
    type Response = ResponsePacket;
    type Error = TransportError;
    type Future = TransportFut<'static>;

    #[inline]
    fn poll_ready(&mut self, _cx: &mut task::Context<'_>) -> task::Poll<Result<(), Self::Error>> {
        // reqwest always returns ok
        task::Poll::Ready(Ok(()))
    }

    #[inline]
    fn call(&mut self, req: RequestPacket) -> Self::Future {
        self.request(req)
    }
}

impl Service<RequestPacket> for &Http<reqwest::Client> {
    type Response = ResponsePacket;
    type Error = TransportError;
    type Future = TransportFut<'static>;

    #[inline]
    fn poll_ready(&mut self, _cx: &mut task::Context<'_>) -> task::Poll<Result<(), Self::Error>> {
        // reqwest always returns ok
        task::Poll::Ready(Ok(()))
    }

    #[inline]
    fn call(&mut self, req: RequestPacket) -> Self::Future {
        self.request(req)
    }
}
