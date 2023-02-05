use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, Timestamp};

use crate::models::{
  ContractID, ContractMetadata, IndexBounds, IndexMetadataView, IndexSlotName, IndexSlotValue,
  IndexUpdate,
};

#[cw_serde]
pub struct InstantiateMsg {
  pub acl_address: Option<Addr>,
  pub default_label: Option<String>,
  pub default_code_id: Option<u64>,
  pub code_ids: Vec<u64>,
  pub indices: Option<Vec<IndexSlotName>>,
}

#[cw_serde]
pub enum ExecuteMsg {
  Create {
    code_id: Option<u64>,
    msg: Binary,
    admin: Option<Addr>,
    label: Option<String>,
    indices: Option<Vec<IndexSlotValue>>,
  },
  InsertIndices {
    values: Vec<IndexSlotValue>,
  },
  UpdateIndices {
    values: Option<Vec<IndexUpdate>>,
  },
  RenameIndex {
    name: IndexSlotName,
  },
}

#[cw_serde]
pub enum QueryMsg {
  Count {},
  Select {
    fields: Option<Vec<String>>,
  },
  Read {
    index: IndexBounds,
    fields: Option<Vec<String>>,
    since: Option<Since>,
    limit: Option<u32>,
    desc: Option<bool>,
    cursor: Option<(String, ContractID)>,
    meta: Option<bool>,
  },
}

#[cw_serde]
pub enum Since {
  Rev(u64),
  Timestamp(Timestamp),
}

#[cw_serde]
pub struct CountResponse {
  pub count: u64,
}

#[cw_serde]
pub struct SelectResponse {
  pub count: Option<u64>,
  pub created_by: Option<Addr>,
  pub default_label: Option<Option<String>>,
  pub default_code_id: Option<u64>,
  pub code_ids: Option<Vec<u64>>,
  pub acl_address: Option<Option<Addr>>,
  pub indices: Option<IndexMetadataView>,
}

#[cw_serde]
pub struct ReadResponse {
  pub page: Vec<ContractStateEnvelope>,
  pub cursor: Option<(String, ContractID)>,
  pub count: u8,
}

#[cw_serde]
pub enum ImplementorQueryMsg {
  Select { fields: Option<Vec<String>> },
}

#[cw_serde]
pub struct ContractStateEnvelope {
  pub address: Addr,
  pub meta: Option<ContractMetadata>,
  pub state: Option<Binary>,
}
