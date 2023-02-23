use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, Timestamp};

pub const SLOT_COUNT: u8 = 5;

pub type ContractID = u64;
pub type IndexTypeCode = u8;
pub type Slot = u8;
pub type Cursor = (String, ContractID);

#[cw_serde]
pub struct ContractMetadata {
  pub id: ContractID,
  pub code_id: u64,
  pub height: u64,
  pub created_at: Timestamp,
  pub updated_at: Timestamp,
  pub rev: u64,
}

#[cw_serde]
pub struct IndexMetadata {
  pub slot: Slot,
  pub name: Option<String>,
  pub updated_at: Option<Timestamp>,
  pub updated_key: Option<(IndexPrefix, ContractID)>,
  pub size: u64,
}

impl IndexMetadata {
  pub fn new(
    slot: Slot,
    name: &Option<String>,
  ) -> Self {
    Self {
      name: name.clone(),
      updated_at: None,
      updated_key: None,
      size: 0,
      slot,
    }
  }
}

#[cw_serde]
pub struct InstantiationPreset {
  pub code_id: Option<u64>,
  pub msg: Binary,
  pub admin: Option<Addr>,
  pub indices: Option<Vec<IndexSlotValue>>,
  pub label: Option<String>,
  pub tags: Option<Vec<String>>,
}

#[cw_serde]
#[derive(Eq, Hash)]
pub struct AddressTag {
  pub address: Addr,
  pub tag: String,
}

#[cw_serde]
pub struct TagUpdates {
  pub added: Option<Vec<String>>,
  pub removed: Option<Vec<String>>,
}

#[cw_serde]
pub struct RelationshipUpdates {
  pub added: Option<Vec<AddressTag>>,
  pub removed: Option<Vec<AddressTag>>,
}

#[cw_serde]
pub struct KeyValue {
  pub key: String,
  pub value: String,
}

#[cw_serde]
pub enum IndexPrefix {
  Uint64(u64),
  Uint128(u128),
  Text(String),
  Timestamp(u64),
  Boolean(u8),
}

#[cw_serde]
pub enum IndexSlotValue {
  Uint64 { slot: Slot, value: u64 },
  Uint128 { slot: Slot, value: u128 },
  Timestamp { slot: Slot, value: Timestamp },
  Text { slot: Slot, value: String },
  Boolean { slot: Slot, value: bool },
}

#[cw_serde]
pub enum IndexSlotName {
  Uint64 { slot: Slot, name: Option<String> },
  Uint128 { slot: Slot, name: Option<String> },
  Timestamp { slot: Slot, name: Option<String> },
  Text { slot: Slot, name: Option<String> },
  Boolean { slot: Slot, name: Option<String> },
}

#[cw_serde]
pub struct IndexedValues {
  pub uint64: Vec<Option<u64>>,
  pub uint128: Vec<Option<u128>>,
  pub text: Vec<Option<String>>,
  pub timestamp: Vec<Option<u64>>,
  pub boolean: Vec<Option<u8>>,
}

impl IndexedValues {
  pub fn new() -> Self {
    let mut uint64: Vec<Option<u64>> = Vec::with_capacity(SLOT_COUNT as usize);
    let mut uint128: Vec<Option<u128>> = Vec::with_capacity(SLOT_COUNT as usize);
    let mut text: Vec<Option<String>> = Vec::with_capacity(SLOT_COUNT as usize);
    let mut timestamp: Vec<Option<u64>> = Vec::with_capacity(SLOT_COUNT as usize);
    let mut boolean: Vec<Option<u8>> = Vec::with_capacity(SLOT_COUNT as usize);
    uint64.resize_with(SLOT_COUNT as usize, Option::default);
    uint128.resize_with(SLOT_COUNT as usize, Option::default);
    text.resize_with(SLOT_COUNT as usize, Option::default);
    timestamp.resize_with(SLOT_COUNT as usize, Option::default);
    boolean.resize_with(SLOT_COUNT as usize, Option::default);
    Self {
      uint64,
      uint128,
      text,
      timestamp,
      boolean,
    }
  }
}

#[cw_serde]
pub enum IndexSlotNameValue {
  Uint64 {
    slot: Slot,
    name: Option<String>,
    value: Option<u64>,
  },
  Uint128 {
    slot: Slot,
    name: Option<String>,
    value: Option<u128>,
  },
  Timestamp {
    slot: Slot,
    name: Option<String>,
    value: Option<Timestamp>,
  },
  Text {
    slot: Slot,
    name: Option<String>,
    value: Option<String>,
  },
  Boolean {
    slot: Slot,
    name: Option<String>,
    value: Option<bool>,
  },
}

#[cw_serde]
pub enum IndexBounds {
  CodeId {
    between: Option<(Option<u64>, Option<u64>)>,
    equals: Option<u64>,
  },
  Height {
    between: Option<(Option<u64>, Option<u64>)>,
    equals: Option<u64>,
  },
  Address {
    between: Option<(Option<Addr>, Option<Addr>)>,
    equals: Option<Addr>,
  },
  CreatedBy {
    between: Option<(Option<Addr>, Option<Addr>)>,
    equals: Option<Addr>,
  },
  CreatedAt {
    between: Option<(Option<Timestamp>, Option<Timestamp>)>,
    equals: Option<Timestamp>,
  },
  UpdatedAt {
    between: Option<(Option<Timestamp>, Option<Timestamp>)>,
    equals: Option<Timestamp>,
  },
  Rev {
    between: Option<(Option<u64>, Option<u64>)>,
    equals: Option<u64>,
  },
  Uint64 {
    slot: u8,
    between: Option<(Option<u64>, Option<u64>)>,
    equals: Option<u64>,
  },
  Uint128 {
    slot: u8,
    between: Option<(Option<u128>, Option<u128>)>,
    equals: Option<u128>,
  },
  Timestamp {
    slot: u8,
    between: Option<(Option<Timestamp>, Option<Timestamp>)>,
    equals: Option<Timestamp>,
  },
  Text {
    slot: u8,
    between: Option<(Option<String>, Option<String>)>,
    equals: Option<String>,
  },
  Boolean {
    slot: u8,
    start: Option<bool>,
    stop: Option<bool>,
  },
}

#[cw_serde]
pub struct IndexMetadataView {
  pub uint64: Vec<IndexMetadata>,
  pub uint128: Vec<IndexMetadata>,
  pub text: Vec<IndexMetadata>,
  pub timestamp: Vec<IndexMetadata>,
  pub boolean: Vec<IndexMetadata>,
}

#[cw_serde]
pub struct Select {
  pub fields: Option<Vec<String>>,
}
