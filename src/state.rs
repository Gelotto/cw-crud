use crate::models::{
  ContractMetadata, IndexMetadata, IndexSlotName, IndexedValues, InstantiationPreset, Slot,
};
use crate::msg::InstantiateMsg;
use crate::{error::ContractError, models::ContractID};
use cosmwasm_std::{Addr, DepsMut, Empty, Env, MessageInfo, QuerierWrapper, Response, Storage};
use cw_acl::client::Acl;
use cw_storage_plus::{Item, Map};

pub type Uint64IndexMap<'a> = Map<'a, (u64, ContractID), bool>;
pub type Uint128IndexMap<'a> = Map<'a, (u128, ContractID), bool>;
pub type TextIndexMap<'a> = Map<'a, (String, ContractID), bool>;
pub type AddrIndexMap<'a> = Map<'a, (Addr, ContractID), bool>;
pub type BoolIndexMap<'a> = Map<'a, (u8, ContractID), bool>;

/// Identity of repo's creator
pub const CREATED_BY: Item<Addr> = Item::new("created_by");

/// whitelist for code ID's allowed instantiation via the create API
pub const ALLOWED_CODE_IDS: Map<u64, bool> = Map::new("allowed_code_ids");

/// Default label of contracts instantiated via the create API
pub const DEFAULT_LABEL: Item<Option<String>> = Item::new("default_label");

/// Default code ID to use in create if not overriden by param
pub const DEFAULT_CODE_ID: Item<u64> = Item::new("default_code_id");

/// Total number of contracts in this repo
pub const COUNT: Item<u64> = Item::new("count");

/// Address for ACL contract used by this repo
pub const ACL_CONTRACT_ADDR: Item<Option<Addr>> = Item::new("acl_contract_addr");

/// named presets stored for instantiating tx sender
pub const PRESETS: Map<(Addr, String), InstantiationPreset> = Map::new("presets");

/// RELATIONSHIPS is used to enable querying contracts associated with a given
/// wallet address through a relationship name.
pub const RELATIONSHIPS: Map<(Addr, String, ContractID), bool> = Map::new("relationships");

/// TAGGED_ADDRESSES is for looking up contract addresses by string tag
pub const TAGGED_CONTRACT_IDS: Map<(String, ContractID), bool> = Map::new("tagged_contract_ids");

/// Lookup table from contract ID to addr
pub const ID_2_ADDR: Map<ContractID, Addr> = Map::new("id_2_addr");

/// Lookup table from contract addr to ID
pub const ADDR_2_ID: Map<Addr, ContractID> = Map::new("addr_2_id");

/// Lookup table from contract ID to vecs of optional values to use as
/// prefixes when looking up the entry in a custom index below
pub const ID_2_INDEXED_VALUES: Map<ContractID, IndexedValues> = Map::new("id_2_indexed_values");

/// Metadata stored for each contract in this repo
pub const METADATA: Map<Addr, ContractMetadata> = Map::new("contract_metadata");

/// Metadata storage for each custom index
pub const IX_META_U64: Map<Slot, IndexMetadata> = Map::new("u64_index_metadata");
pub const IX_META_U128: Map<Slot, IndexMetadata> = Map::new("u128_index_metadata");
pub const IX_META_STRING: Map<Slot, IndexMetadata> = Map::new("text_index_metadata");
pub const IX_META_BOOL: Map<Slot, IndexMetadata> = Map::new("bool_index_metadata");
pub const IX_META_TIMESTAMP: Map<Slot, IndexMetadata> = Map::new("ts_index_metadata");

/// Built-in indices
pub const IX_CREATED_BY: AddrIndexMap = Map::new("ix_created_by");
pub const IX_CREATED_AT: Uint64IndexMap = Map::new("ix_created_at");
pub const IX_UPDATED_AT: Uint64IndexMap = Map::new("ix_updated_at");
pub const IX_CODE_ID: Uint64IndexMap = Map::new("ix_code_id");
pub const IX_HEIGHT: Uint64IndexMap = Map::new("ix_height");
pub const IX_REV: Uint64IndexMap = Map::new("ix_rev");

/// Custom index slots
pub const IX_U64_0: Uint64IndexMap = Map::new("ix_u64_0");
pub const IX_U64_1: Uint64IndexMap = Map::new("ix_u64_1");
pub const IX_U64_2: Uint64IndexMap = Map::new("ix_u64_2");
pub const IX_U64_3: Uint64IndexMap = Map::new("ix_u64_3");
pub const IX_U64_4: Uint64IndexMap = Map::new("ix_u64_4");

pub const IX_U128_0: Uint128IndexMap = Map::new("ix_u128_0");
pub const IX_U128_1: Uint128IndexMap = Map::new("ix_u128_1");
pub const IX_U128_2: Uint128IndexMap = Map::new("ix_u128_2");
pub const IX_U128_3: Uint128IndexMap = Map::new("ix_u128_3");
pub const IX_U128_4: Uint128IndexMap = Map::new("ix_u128_4");

pub const IX_TS_0: Uint64IndexMap = Map::new("ix_ts_u64_0");
pub const IX_TS_1: Uint64IndexMap = Map::new("ix_ts_u64_1");
pub const IX_TS_2: Uint64IndexMap = Map::new("ix_ts_u64_2");
pub const IX_TS_3: Uint64IndexMap = Map::new("ix_ts_u64_3");
pub const IX_TS_4: Uint64IndexMap = Map::new("ix_ts_u64_4");

pub const IX_BOOL_0: BoolIndexMap = Map::new("ix_bool_0");
pub const IX_BOOL_1: BoolIndexMap = Map::new("ix_bool_1");
pub const IX_BOOL_2: BoolIndexMap = Map::new("ix_bool_2");
pub const IX_BOOL_3: BoolIndexMap = Map::new("ix_bool_3");
pub const IX_BOOL_4: BoolIndexMap = Map::new("ix_bool_4");

pub const IX_TEXT_0: TextIndexMap = Map::new("ix_text_0");
pub const IX_TEXT_1: TextIndexMap = Map::new("ix_text_1");
pub const IX_TEXT_2: TextIndexMap = Map::new("ix_text_2");
pub const IX_TEXT_3: TextIndexMap = Map::new("ix_text_3");
pub const IX_TEXT_4: TextIndexMap = Map::new("ix_text_4");

/// Initialize contract state.
pub fn initialize(
  deps: DepsMut,
  _env: &Env,
  info: &MessageInfo,
  msg: &InstantiateMsg,
) -> Result<Response, ContractError> {
  if msg.code_ids.is_empty() {
    return Err(ContractError::NoAllowedCodeIds {});
  }

  if let Some(default_label) = &msg.default_label {
    if default_label.is_empty() {
      return Err(ContractError::InvalidLabel {});
    }
  }

  // if no default code ID given, set to the first element of allowed code IDs
  let default_code_id = msg.default_code_id.unwrap_or(msg.code_ids[0]);

  if !msg.code_ids.contains(&default_code_id) {
    deps
      .api
      .debug(format!("cannot set default code ID if not in allowed code ID vec").as_str());
    return Err(ContractError::CodeIdNotAllowed {});
  }

  CREATED_BY.save(deps.storage, &info.sender)?;
  ACL_CONTRACT_ADDR.save(deps.storage, &msg.acl_address)?;
  DEFAULT_LABEL.save(deps.storage, &msg.default_label)?;
  DEFAULT_CODE_ID.save(deps.storage, &default_code_id)?;
  COUNT.save(deps.storage, &0)?;

  for code_id in msg.code_ids.iter() {
    ALLOWED_CODE_IDS.save(deps.storage, *code_id, &true)?;
  }

  if let Some(indices) = &msg.indices {
    for x in indices.iter() {
      match x {
        IndexSlotName::Uint64 { slot, name } => {
          IX_META_U64.save(deps.storage, *slot, &IndexMetadata::new(*slot, name))?
        },
        IndexSlotName::Uint128 { slot, name } => {
          IX_META_U128.save(deps.storage, *slot, &IndexMetadata::new(*slot, name))?
        },
        IndexSlotName::Timestamp { slot, name } => {
          IX_META_TIMESTAMP.save(deps.storage, *slot, &IndexMetadata::new(*slot, name))?
        },
        IndexSlotName::Text { slot, name } => {
          IX_META_STRING.save(deps.storage, *slot, &IndexMetadata::new(*slot, name))?
        },
        IndexSlotName::Boolean { slot, name } => {
          IX_META_BOOL.save(deps.storage, *slot, &IndexMetadata::new(*slot, name))?
        },
      }
    }
  }

  Ok(Response::new().add_attribute("action", "instantiate"))
}

/// Helper function that returns true if given wallet (principal) is authorized
/// by ACL to the given action. If there's no ACL, we only authorize the sender
/// if it is the created_by address.
pub fn is_allowed(
  storage: &mut dyn Storage,
  querier: &QuerierWrapper<Empty>,
  principal: &Addr,
  action: &str,
) -> Result<bool, ContractError> {
  if let Some(acl_addr) = ACL_CONTRACT_ADDR.load(storage)? {
    let acl = Acl::new(&acl_addr);
    Ok(acl.is_allowed(querier, principal, action)?)
  } else {
    Ok(CREATED_BY.load(storage)? == *principal)
  }
}

/// increment the collection count, returning pre-incremented value.
pub fn get_next_contract_id(storage: &mut dyn Storage) -> Result<u64, ContractError> {
  return Ok(COUNT.update(storage, |n| -> Result<u64, ContractError> { Ok(n + 1) })? - 1);
}

// Was the given contract address created through this contract's `create`?
pub fn owns_contract(
  storage: &dyn Storage,
  contract_addr: &Addr,
) -> bool {
  METADATA.has(storage, contract_addr.clone())
}

pub fn get_contract_id(
  storage: &dyn Storage,
  contract_addr: &Addr,
) -> Result<ContractID, ContractError> {
  if let Some(id) = ADDR_2_ID.may_load(storage, contract_addr.clone())? {
    Ok(id)
  } else {
    Err(ContractError::NotFound {})
  }
}

pub fn get_u64_index(slot: u8) -> Result<Uint64IndexMap<'static>, ContractError> {
  match slot {
    0 => Ok(IX_U64_0),
    1 => Ok(IX_U64_1),
    2 => Ok(IX_U64_2),
    3 => Ok(IX_U64_3),
    4 => Ok(IX_U64_4),
    _ => Err(ContractError::NotAuthorized {}),
  }
}

pub fn get_u128_index(slot: u8) -> Result<Uint128IndexMap<'static>, ContractError> {
  match slot {
    0 => Ok(IX_U128_0),
    1 => Ok(IX_U128_1),
    2 => Ok(IX_U128_2),
    3 => Ok(IX_U128_3),
    4 => Ok(IX_U128_4),
    _ => Err(ContractError::NotAuthorized {}),
  }
}

pub fn get_timestamp_index(slot: u8) -> Result<Uint64IndexMap<'static>, ContractError> {
  match slot {
    0 => Ok(IX_TS_0),
    1 => Ok(IX_TS_1),
    2 => Ok(IX_TS_2),
    3 => Ok(IX_TS_3),
    4 => Ok(IX_TS_4),
    _ => Err(ContractError::NotAuthorized {}),
  }
}

pub fn get_text_index(slot: u8) -> Result<TextIndexMap<'static>, ContractError> {
  match slot {
    0 => Ok(IX_TEXT_0),
    1 => Ok(IX_TEXT_1),
    2 => Ok(IX_TEXT_2),
    3 => Ok(IX_TEXT_3),
    4 => Ok(IX_TEXT_4),
    _ => Err(ContractError::NotAuthorized {}),
  }
}

pub fn get_bool_index(slot: u8) -> Result<BoolIndexMap<'static>, ContractError> {
  match slot {
    0 => Ok(IX_BOOL_0),
    1 => Ok(IX_BOOL_1),
    2 => Ok(IX_BOOL_2),
    3 => Ok(IX_BOOL_3),
    4 => Ok(IX_BOOL_4),
    _ => Err(ContractError::NotAuthorized {}),
  }
}

pub fn increment_index_size<'a>(
  storage: &mut dyn Storage,
  map: &Map<'a, Slot, IndexMetadata>,
  slot: Slot,
) -> Result<IndexMetadata, ContractError> {
  update_index_size(storage, map, slot, 1, true)
}

pub fn decrement_index_size<'a>(
  storage: &mut dyn Storage,
  map: &Map<'a, Slot, IndexMetadata>,
  slot: Slot,
) -> Result<IndexMetadata, ContractError> {
  update_index_size(storage, map, slot, 1, false)
}

fn update_index_size<'a>(
  storage: &mut dyn Storage,
  map: &Map<'a, Slot, IndexMetadata>,
  slot: Slot,
  delta: u64,
  increment: bool,
) -> Result<IndexMetadata, ContractError> {
  Ok(map.update(
    storage,
    slot,
    |some_meta| -> Result<IndexMetadata, ContractError> {
      if let Some(mut meta) = some_meta {
        if increment {
          meta.size += delta;
        } else {
          meta.size -= delta;
        }
        Ok(meta)
      } else {
        // shouldn't be possible to reach this point
        Err(ContractError::InvalidIndexSlot {})
      }
    },
  )?)
}
