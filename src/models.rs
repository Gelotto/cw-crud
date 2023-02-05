use base64::{engine::general_purpose as b64, Engine as _};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp};

pub type ContractID = u64;
pub type IndexTypeCode = u8;
pub type Slot = u8;

#[cw_serde]
pub struct ContractMetadata {
  pub id: ContractID,
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
pub struct KeyValue {
  pub key: String,
  pub value: String,
}

#[cw_serde]
pub enum IndexPrefix {
  Number(u64),
  Text(String),
  Timestamp(u64),
  Boolean(u8),
}

#[cw_serde]
pub enum IndexUpdateValues {
  Number(u64, u64),
  Text(String, String),
  Timestamp(Timestamp, Timestamp),
  Boolean(bool, bool),
}

#[cw_serde]
pub enum IndexSlotValue {
  Number { slot: Slot, value: u64 },
  Timestamp { slot: Slot, value: Timestamp },
  Text { slot: Slot, value: String },
  Boolean { slot: Slot, value: bool },
}

#[cw_serde]
pub enum IndexSlotName {
  Number { slot: Slot, name: Option<String> },
  Timestamp { slot: Slot, name: Option<String> },
  Text { slot: Slot, name: Option<String> },
  Boolean { slot: Slot, name: Option<String> },
}

#[cw_serde]
pub enum IndexSlotNameValue {
  Number {
    slot: Slot,
    name: Option<String>,
    value: Option<u64>,
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
pub struct IndexUpdate {
  pub slot: Slot,
  pub values: IndexUpdateValues,
}

impl IndexUpdate {
  pub fn number(
    slot: Slot,
    old_value: u64,
    new_value: u64,
  ) -> Self {
    Self {
      slot,
      values: IndexUpdateValues::Number(old_value, new_value),
    }
  }

  pub fn timestamp(
    slot: Slot,
    old_value: Timestamp,
    new_value: Timestamp,
  ) -> Self {
    Self {
      slot,
      values: IndexUpdateValues::Number(old_value.nanos(), new_value.nanos()),
    }
  }

  pub fn boolean(
    slot: Slot,
    old_value: bool,
    new_value: bool,
  ) -> Self {
    Self {
      slot,
      values: IndexUpdateValues::Boolean(old_value, new_value),
    }
  }

  pub fn text(
    slot: Slot,
    old_value: String,
    new_value: String,
  ) -> Self {
    Self {
      slot,
      values: IndexUpdateValues::Text(
        b64::STANDARD.encode(&old_value),
        b64::STANDARD.encode(&new_value),
      ),
    }
  }
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
  Number {
    slot: u8,
    between: Option<(Option<u64>, Option<u64>)>,
    equals: Option<u64>,
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
  pub number: Vec<IndexMetadata>,
  pub text: Vec<IndexMetadata>,
  pub timestamp: Vec<IndexMetadata>,
  pub boolean: Vec<IndexMetadata>,
}

#[cw_serde]
pub struct Select {
  pub fields: Option<Vec<String>>,
}
