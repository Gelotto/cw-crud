use cosmwasm_std::{Addr, Timestamp};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub type ContractID = u64;

/// Initial contract state.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContractMetadata {
  pub id: ContractID,
  pub height: u64,
  pub created_at: Timestamp,
  pub updated_at: Timestamp,
  pub rev: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct KeyValue {
  pub key: String,
  pub value: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IndexUpdateValues {
  Numeric(u64, u64),
  Text(String, String),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct IndexUpdate {
  pub index: u8,
  pub values: IndexUpdateValues,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IndexSelection {
  CodeId {
    start: Option<u64>,
    stop: Option<u64>,
  },
  Address {
    start: Option<Addr>,
    stop: Option<Addr>,
  },
  CreatedAt {
    start: Option<Timestamp>,
    stop: Option<Timestamp>,
  },
  UpdatedAt {
    start: Option<Timestamp>,
    stop: Option<Timestamp>,
  },
  Revision {
    start: Option<u64>,
    stop: Option<u64>,
  },
  Numeric {
    idx: u8,
    start: Option<u64>,
    stop: Option<u64>,
  },
  Text {
    idx: u8,
    start: Option<String>,
    stop: Option<String>,
  },
}
