use crate::models::{ContractMetadata, IndexMetadata, IndexSlotName, Slot};
use crate::msg::InstantiateMsg;
use crate::{error::ContractError, models::ContractID};
use cosmwasm_std::{
  Addr, DepsMut, Empty, Env, MessageInfo, QuerierWrapper, Response, StdResult, Storage,
};
use cw_lib::acl::api::is_allowed as is_allowed_by_acl;
use cw_storage_plus::{Item, Map};

pub type NumberIndexMap<'a> = Map<'a, (u64, ContractID), bool>;
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

/// Lookup table from contract ID to addr
pub const ID_2_ADDR: Map<ContractID, Addr> = Map::new("id_2_addr");

/// Lookup table from contract addr to ID
pub const ADDR_2_ID: Map<Addr, ContractID> = Map::new("addr_2_id");

/// Metadata stored for each contract in this repo
pub const METADATA: Map<Addr, ContractMetadata> = Map::new("contract_metadata");

/// Metadata storage for each custom index
pub const NUMBER_INDEX_METADATA: Map<Slot, IndexMetadata> = Map::new("number_index_metadata");
pub const TEXT_INDEX_METADATA: Map<Slot, IndexMetadata> = Map::new("text_index_metadata");
pub const BOOL_INDEX_METADATA: Map<Slot, IndexMetadata> = Map::new("bool_index_metadata");
pub const TS_INDEX_METADATA: Map<Slot, IndexMetadata> = Map::new("ts_index_metadata");

/// Built-in indices
pub const IX_CREATED_BY: AddrIndexMap = Map::new("ix_created_by");
pub const IX_CREATED_AT: NumberIndexMap = Map::new("ix_created_at");
pub const IX_UPDATED_AT: NumberIndexMap = Map::new("ix_updated_at");
pub const IX_CODE_ID: NumberIndexMap = Map::new("ix_code_id");
pub const IX_HEIGHT: NumberIndexMap = Map::new("ix_height");
pub const IX_REV: NumberIndexMap = Map::new("ix_rev");

/// Custom index slots
// TODO: add seperate maps for None values
pub const IX_NUMBER_0: NumberIndexMap = Map::new("ix_number_0");
pub const IX_NUMBER_1: NumberIndexMap = Map::new("ix_number_1");
pub const IX_NUMBER_2: NumberIndexMap = Map::new("ix_number_2");
pub const IX_NUMBER_3: NumberIndexMap = Map::new("ix_number_3");
pub const IX_NUMBER_4: NumberIndexMap = Map::new("ix_number_4");

pub const IX_TS_0: NumberIndexMap = Map::new("ix_ts_u64_0");
pub const IX_TS_1: NumberIndexMap = Map::new("ix_ts_u64_1");
pub const IX_TS_2: NumberIndexMap = Map::new("ix_ts_u64_2");
pub const IX_TS_3: NumberIndexMap = Map::new("ix_ts_u64_3");
pub const IX_TS_4: NumberIndexMap = Map::new("ix_ts_u64_4");

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
        IndexSlotName::Number { slot, name } => {
          NUMBER_INDEX_METADATA.save(deps.storage, *slot, &IndexMetadata::new(*slot, name))?
        },
        IndexSlotName::Timestamp { slot, name } => {
          TS_INDEX_METADATA.save(deps.storage, *slot, &IndexMetadata::new(*slot, name))?
        },
        IndexSlotName::Text { slot, name } => {
          TEXT_INDEX_METADATA.save(deps.storage, *slot, &IndexMetadata::new(*slot, name))?
        },
        IndexSlotName::Boolean { slot, name } => {
          BOOL_INDEX_METADATA.save(deps.storage, *slot, &IndexMetadata::new(*slot, name))?
        },
      }
    }
  }

  Ok(Response::new().add_attribute("action", "instantiate"))
}

/// Helper function that returns true if given wallet (principal) is authorized
/// by ACL to the given action.
pub fn is_allowed(
  storage: &mut dyn Storage,
  querier: &QuerierWrapper<Empty>,
  principal: &Addr,
  action: &str,
) -> Result<bool, ContractError> {
  if let Some(acl_addr) = ACL_CONTRACT_ADDR.load(storage)? {
    if !is_allowed_by_acl(querier, &acl_addr, principal, action)? {
      return Ok(false);
    }
  }
  Ok(true)
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
) -> StdResult<ContractID> {
  ADDR_2_ID.load(storage, contract_addr.clone())
}

pub fn get_number_index(slot: u8) -> Result<NumberIndexMap<'static>, ContractError> {
  match slot {
    0 => Ok(IX_NUMBER_0),
    1 => Ok(IX_NUMBER_1),
    2 => Ok(IX_NUMBER_2),
    3 => Ok(IX_NUMBER_3),
    4 => Ok(IX_NUMBER_4),
    _ => Err(ContractError::NotAuthorized {}),
  }
}

pub fn get_timestamp_index(slot: u8) -> Result<NumberIndexMap<'static>, ContractError> {
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

pub fn update_index_metadata_count<'a>(
  storage: &mut dyn Storage,
  map: &Map<'a, Slot, IndexMetadata>,
  slot: Slot,
) -> Result<IndexMetadata, ContractError> {
  Ok(map.update(
    storage,
    slot,
    |some_meta| -> Result<IndexMetadata, ContractError> {
      if let Some(mut meta) = some_meta {
        meta.size += 1;
        Ok(meta)
      } else {
        // shouldn't be possible to reach this point
        Err(ContractError::InvalidIndexSlot {})
      }
    },
  )?)
}
