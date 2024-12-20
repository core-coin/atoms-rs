//! Core types for pub-sub

use crate::{
    eth::{Filter, Transaction},
    Log, RichHeader,
};
use base_primitives::B256;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

/// Subscription result.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum SubscriptionResult {
    /// New block header.
    Header(Box<RichHeader>),
    /// Log
    Log(Box<Log>),
    /// Transaction hash
    TransactionHash(B256),
    /// Full Transaction
    FullTransaction(Box<Transaction>),
    /// SyncStatus
    SyncState(PubSubSyncStatus),
}

/// Response type for a SyncStatus subscription.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PubSubSyncStatus {
    /// If not currently syncing, this should always be `false`.
    Simple(bool),
    /// Syncing metadata.
    Detailed(SyncStatusMetadata),
}

/// Sync status metadata.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncStatusMetadata {
    /// Whether the node is currently syncing.
    pub syncing: bool,
    /// The starting block.
    pub starting_block: u64,
    /// The current block.
    pub current_block: u64,
    /// The highest block.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub highest_block: Option<u64>,
}

impl Serialize for SubscriptionResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            Self::Header(ref header) => header.serialize(serializer),
            Self::Log(ref log) => log.serialize(serializer),
            Self::TransactionHash(ref hash) => hash.serialize(serializer),
            Self::FullTransaction(ref tx) => tx.serialize(serializer),
            Self::SyncState(ref sync) => sync.serialize(serializer),
        }
    }
}

/// Subscription kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub enum SubscriptionKind {
    /// New block headers subscription.
    ///
    /// Fires a notification each time a new header is appended to the chain, including chain
    /// reorganizations. In case of a chain reorganization the subscription will emit all new
    /// headers for the new chain. Therefore the subscription can emit multiple headers on the same
    /// height.
    NewHeads,
    /// Logs subscription.
    ///
    /// Returns logs that are included in new imported blocks and match the given filter criteria.
    /// In case of a chain reorganization previous sent logs that are on the old chain will be
    /// resent with the removed property set to true. Logs from transactions that ended up in the
    /// new chain are emitted. Therefore, a subscription can emit logs for the same transaction
    /// multiple times.
    Logs,
    /// New Pending Transactions subscription.
    ///
    /// Returns the hash or full tx for all transactions that are added to the pending state and
    /// are signed with a key that is available in the node. When a transaction that was
    /// previously part of the canonical chain isn't part of the new canonical chain after a
    /// reorganization its again emitted.
    NewPendingTransactions,
    /// Node syncing status subscription.
    ///
    /// Indicates when the node starts or stops synchronizing. The result can either be a boolean
    /// indicating that the synchronization has started (true), finished (false) or an object with
    /// various progress indicators.
    Syncing,
}

/// Any additional parameters for a subscription.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Params {
    /// No parameters passed.
    #[default]
    None,
    /// Log parameters.
    Logs(Box<Filter>),
    /// Boolean parameter for new pending transactions.
    Bool(bool),
}

impl Params {
    /// Returns true if it's a bool parameter.
    #[inline]
    pub const fn is_bool(&self) -> bool {
        matches!(self, Self::Bool(_))
    }

    /// Returns true if it's a log parameter.
    #[inline]
    pub const fn is_logs(&self) -> bool {
        matches!(self, Self::Logs(_))
    }
}

impl Serialize for Params {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::None => (&[] as &[serde_json::Value]).serialize(serializer),
            Self::Logs(logs) => logs.serialize(serializer),
            Self::Bool(full) => full.serialize(serializer),
        }
    }
}

impl<'a> Deserialize<'a> for Params {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a>,
    {
        let v = serde_json::Value::deserialize(deserializer)?;

        if v.is_null() {
            return Ok(Self::None);
        }

        if let Some(val) = v.as_bool() {
            return Ok(Self::Bool(val));
        }

        serde_json::from_value(v)
            .map(|f| Self::Logs(Box::new(f)))
            .map_err(|e| D::Error::custom(format!("Invalid Pub-Sub parameters: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn params_serde() {
        let s: Params = serde_json::from_str("true").unwrap();
        assert_eq!(s, Params::Bool(true));
        let s: Params = serde_json::from_str("null").unwrap();
        assert_eq!(s, Params::None);
    }
}
