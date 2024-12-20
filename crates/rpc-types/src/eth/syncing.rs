use base_primitives::{U256, U64};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;
use libgoldilocks::VerifyingKey;

/// Syncing info
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncInfo {
    /// Starting block
    pub starting_block: U256,
    /// Current block
    pub current_block: U256,
    /// Highest block seen so far
    pub highest_block: U256,
    /// Warp sync snapshot chunks total.
    pub warp_chunks_amount: Option<U256>,
    /// Warp sync snapshot chunks processed.
    pub warp_chunks_processed: Option<U256>,
}

/// Peers info
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Peers {
    /// Number of active peers
    pub active: usize,
    /// Number of connected peers
    pub connected: usize,
    /// Max number of peers
    pub max: u32,
    /// Detailed information on peers
    pub peers: Vec<PeerInfo>,
}

/// Number of peers connected to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PeerCount {
    /// Peer count as integer
    Number(u32),
    /// Peer count as hex
    Hex(U64),
}

/// Peer connection information
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerInfo {
    /// Public node id
    pub id: Option<String>,
    /// Node client ID
    pub name: String,
    /// Capabilities
    pub caps: Vec<String>,
    /// Network information
    pub network: PeerNetworkInfo,
    /// Protocols information
    pub protocols: PeerProtocolsInfo,
}

/// Peer network information
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PeerNetworkInfo {
    /// Remote endpoint address
    pub remote_address: String,
    /// Local endpoint address
    pub local_address: String,
}

/// Peer protocols information
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerProtocolsInfo {
    /// Core protocol information
    pub xcb: Option<PeerXcbProtocolInfo>,
    /// PIP protocol information.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pip: Option<PipProtocolInfo>,
}

/// Peer Core protocol information
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerXcbProtocolInfo {
    /// Negotiated core protocol version
    pub version: u32,
    /// Peer total difficulty if known
    pub difficulty: Option<U256>,
    /// SHA3 of peer best block hash
    pub head: String,
}

/// Peer PIP protocol information
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PipProtocolInfo {
    /// Negotiated PIP protocol version
    pub version: u32,
    /// Peer total difficulty
    pub difficulty: U256,
    /// SHA3 of peer best block hash
    pub head: String,
}

/// Sync status
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyncStatus {
    /// Info when syncing
    Info(SyncInfo),
    /// Not syncing
    None,
}

impl<'de> Deserialize<'de> for SyncStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Syncing {
            /// When client is synced to the highest block, xcb_syncing with return "false"
            None(bool),
            IsSyncing(SyncInfo),
        }

        match Syncing::deserialize(deserializer)? {
            Syncing::None(false) => Ok(SyncStatus::None),
            Syncing::None(true) => Err(serde::de::Error::custom(
                "xcb_syncing returned `true` that is undefined value.",
            )),
            Syncing::IsSyncing(sync) => Ok(SyncStatus::Info(sync)),
        }
    }
}

impl Serialize for SyncStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            SyncStatus::Info(info) => info.serialize(serializer),
            SyncStatus::None => serializer.serialize_bool(false),
        }
    }
}

/// Propagation statistics for pending transaction.
#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionStats {
    /// Block no this transaction was first seen.
    pub first_seen: u64,
    /// Peers this transaction was propagated to with count.
    pub propagated_to: BTreeMap<VerifyingKey, usize>,
}

/// Chain status.
#[derive(Clone, Copy, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainStatus {
    /// Describes the gap in the blockchain, if there is one: (first, last)
    pub block_gap: Option<(U256, U256)>,
}
