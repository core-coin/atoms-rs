//! Types for opcode tracing.

use alloy_primitives::B256;
use serde::{Deserialize, Serialize};

/// Opcode energy usage for a transaction.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockOpcodeEnergy {
    /// The block hash
    pub block_hash: B256,
    /// The block number
    pub block_number: u64,
    /// All executed transactions in the block in the order they were executed, with their opcode
    /// energy usage.
    pub transactions: Vec<TransactionOpcodeEnergy>,
}

/// Opcode energy usage for a transaction.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionOpcodeEnergy {
    /// The transaction hash
    pub transaction_hash: B256,
    /// The energy used by each opcode in the transaction
    pub opcode_energy: Vec<OpcodeEnergy>,
}

/// Energy information for a single opcode.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpcodeEnergy {
    /// The name of the opcode
    pub opcode: String,
    /// How many times the opcode was executed
    pub count: u64,
    /// Combined energy used by all instances of the opcode
    ///
    /// For opcodes with constant energy costs, this is the constant opcode energy cost times the count.
    pub energy_used: u64,
}
