use cosmwasm_std::{Addr, Binary, Coin, Timestamp};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::models::{ContractMetadata, IndexSelection, IndexUpdate};

/// Initial contract state.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
  pub default_label: String,
  pub acl_address: Option<Addr>,
  pub allowed_code_ids: Vec<u64>,
}

/// Initial contract state.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ClientInstantiateMsg {}

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
  },
  Update {
    views: Option<Vec<IndexUpdate>>,
  },
}

/// Custom contract query endpoints.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
  Count {},
  Read {
    index: IndexSelection,
    modified_since: Option<Timestamp>,
    limit: Option<u32>,
    desc: Option<bool>,
    params: Option<Binary>,
    verbose: Option<bool>,
  },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CountResponse {
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
pub struct GetState {
  pub params: Binary,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContractStateEnvelope {
  pub address: Addr,
  pub meta: Option<ContractMetadata>,
  pub state: Option<Binary>,
}
