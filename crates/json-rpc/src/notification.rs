use crate::{Response, ResponsePayload};
use base_primitives::U256;
use serde::{
    de::{MapAccess, Visitor},
    Deserialize, Serialize,
};

/// Core-style notification, not to be confused with a JSON-RPC
/// notification.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct XcbNotification<T = Box<serde_json::value::RawValue>> {
    /// The subscription ID.
    pub subscription: U256,
    /// The notification payload.
    pub result: T,
}

/// An item received over an Core pubsub transport. Core pubsub uses a
/// non-standard JSON-RPC notification format. An item received over a pubsub
/// transport may be a JSON-RPC response or Corestyle notification.
#[derive(Clone, Debug)]
pub enum PubSubItem {
    /// A [`Response`] to a JSON-RPC request.
    Response(Response),
    /// Core-style notification.
    Notification(XcbNotification),
}

impl From<Response> for PubSubItem {
    fn from(response: Response) -> Self {
        Self::Response(response)
    }
}

impl From<XcbNotification> for PubSubItem {
    fn from(notification: XcbNotification) -> Self {
        Self::Notification(notification)
    }
}

impl<'de> Deserialize<'de> for PubSubItem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct PubSubItemVisitor;

        impl<'de> Visitor<'de> for PubSubItemVisitor {
            type Value = PubSubItem;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a JSON-RPC response or Core-style notification")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut id = None;
                let mut result = None;
                let mut params = None;
                let mut error = None;

                // Drain the map into the appropriate fields.
                while let Ok(Some(key)) = map.next_key() {
                    match key {
                        "id" => {
                            if id.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id = Some(map.next_value()?);
                        }
                        "result" => {
                            if result.is_some() {
                                return Err(serde::de::Error::duplicate_field("result"));
                            }
                            result = Some(map.next_value()?);
                        }
                        "params" => {
                            if params.is_some() {
                                return Err(serde::de::Error::duplicate_field("params"));
                            }
                            params = Some(map.next_value()?);
                        }
                        "error" => {
                            if error.is_some() {
                                return Err(serde::de::Error::duplicate_field("error"));
                            }
                            error = Some(map.next_value()?);
                        }
                        // Discard unknown fields.
                        _ => {
                            let _ = map.next_value::<serde_json::Value>()?;
                        }
                    }
                }

                // If it has an ID, it is a response.
                if let Some(id) = id {
                    let payload = error
                        .map(ResponsePayload::Failure)
                        .or_else(|| result.map(ResponsePayload::Success))
                        .ok_or_else(|| {
                            serde::de::Error::custom(
                                "missing `result` or `error` field in response",
                            )
                        })?;

                    Ok(Response { id, payload }.into())
                } else {
                    // Notifications cannot have an error.
                    if error.is_some() {
                        return Err(serde::de::Error::custom(
                            "unexpected `error` field in subscription notification",
                        ));
                    }
                    params
                        .map(PubSubItem::Notification)
                        .ok_or_else(|| serde::de::Error::missing_field("params"))
                }
            }
        }

        deserializer.deserialize_any(PubSubItemVisitor)
    }
}

#[cfg(test)]
mod test {

    use crate::{XcbNotification, PubSubItem};

    #[test]
    fn deserializer_test() {
        let notification = r#"{ "jsonrpc": "2.0", "method": "xcb_subscription", "params": {"subscription": "0xcd0c3e8af590364c09d0fa6a1210faf5", "result": {"difficulty": "0xd9263f42a87", "uncles": []}} }
        "#;

        let deser = serde_json::from_str::<PubSubItem>(notification).unwrap();

        match deser {
            PubSubItem::Notification(XcbNotification { subscription, result }) => {
                assert_eq!(subscription, "0xcd0c3e8af590364c09d0fa6a1210faf5".parse().unwrap());
                assert_eq!(result.get(), r#"{"difficulty": "0xd9263f42a87", "uncles": []}"#);
            }
            _ => panic!("unexpected deserialization result"),
        }
    }
}
