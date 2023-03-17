use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, Timestamp};

use crate::models::{
  AddressTag, ContractID, ContractMetadata, IndexBounds, IndexMetadataView, IndexSlotName,
  IndexSlotValue, IndexedValues, InstantiationPreset, RelationshipUpdates, TagUpdates,
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
    msg: Option<Binary>,
    admin: Option<Addr>,
    label: Option<String>,
    indices: Option<Vec<IndexSlotValue>>,
    preset: Option<String>,
    save_as: Option<String>,
    tags: Option<Vec<String>>,
    relationships: Option<Vec<AddressTag>>,
  },
  RemovePreset {
    preset: String,
  },
  Update {
    values: Option<Vec<IndexSlotValue>>,
    relationships: Option<RelationshipUpdates>,
    tags: Option<TagUpdates>,
  },
  Remove {
    contract_addr: Addr,
  },
  RenameIndex {
    name: IndexSlotName,
  },
  SetAcl {
    acl_contract_addr: Addr,
  },
  UpdateAllowedCodeIds {
    code_ids: Vec<u64>,
  },
}

#[cw_serde]
pub enum Target {
  Index(IndexBounds),
  Relationship((Addr, String)), // subject addr, rel name
  Tag(String),                  // tag associated with one or more contracts
}

#[cw_serde]
pub enum QueryMsg {
  Count {},
  Read {
    target: Target,
    fields: Option<Vec<String>>,
    since: Option<Since>,
    limit: Option<u32>,
    desc: Option<bool>,
    cursor: Option<(String, ContractID)>,
    meta: Option<bool>,
    wallet: Option<Addr>,
  },
  Select {
    wallet: Option<Addr>,
    fields: Option<Vec<String>>,
  },
  Values {
    contract_addr: Addr,
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
pub struct ValuesResponse {
  pub values: IndexedValues,
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
  pub presets: Option<Vec<InstantiationPreset>>,
}

#[cw_serde]
pub struct Page {
  pub page: Vec<EntityContractEnvelope>,
  pub cursor: Option<(String, ContractID)>,
}

#[cw_serde]
pub enum ImplementorQueryMsg {
  Select {
    wallet: Option<Addr>,
    fields: Option<Vec<String>>,
  },
}

#[cw_serde]
pub struct EntityContractEnvelope {
  pub address: Addr,
  pub meta: Option<ContractMetadata>,
  pub state: Option<Binary>,
}
