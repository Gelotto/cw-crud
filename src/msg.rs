use cosmwasm_std::{Addr, Binary, Coin, Timestamp};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::models::{ContractMetadata, IndexInitializationParams, IndexSelection, IndexUpdate};

/// Initial contract state.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
  pub default_label: String,
  pub acl_address: Option<Addr>,
  pub allowed_code_ids: Vec<u64>,
}

/// Executable contract endpoints.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
  Create {
    code_id: u64,
    instantiate_msg: Binary,
    funds: Option<Vec<Coin>>,
    admin: Option<Addr>,
    label: Option<String>,
    indices: Option<Vec<IndexInitializationParams>>,
  },
  Update {
    indices: Option<Vec<IndexUpdate>>,
  },
}

/// Custom contract query endpoints.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
  Count {},
  Select(Select),
  ExecuteSelect {
    index: IndexSelection,
    include: Option<Vec<String>>,
    since: Option<Since>,
    limit: Option<u32>,
    desc: Option<bool>,
    meta: Option<bool>,
  },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Since {
  Rev(u64),
  Timestamp(Timestamp),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CountResponse {
  pub count: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SelectResponse {
  pub created_by: Addr,
  pub default_label: String,
  pub allowed_code_ids: Vec<u64>,
  pub acl_address: Option<Addr>,
  pub count: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ReadResponse {
  pub page: Vec<ContractStateEnvelope>,
  pub count: u8,
  pub start: Option<String>,
  pub stop: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Select {
  pub fields: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContractStateEnvelope {
  pub address: Addr,
  pub meta: Option<ContractMetadata>,
  pub state: Option<Binary>,
}
