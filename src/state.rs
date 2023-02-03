use crate::models::ContractMetadata;
use crate::msg::InstantiateMsg;
use crate::{error::ContractError, models::ContractID};
use cosmwasm_std::{
  Addr, DepsMut, Empty, Env, MessageInfo, QuerierWrapper, Response, StdResult, Storage,
};
use cw_lib::acl::api::is_allowed as is_allowed_by_acl;
use cw_storage_plus::{Item, Map};

pub type U64IndexMap<'a> = Map<'a, (u64, ContractID), bool>;
pub type StrIndexMap<'a> = Map<'a, (String, ContractID), bool>;

/// Identity of repo's creator
pub const CREATED_BY: Item<Addr> = Item::new("created_by");

/// whitelist for code ID's allowed instantiation via the create API
pub const ALLOWED_CODE_IDS: Map<u64, bool> = Map::new("allowed_code_ids");

/// Default label of contracts instantiated via the create API
pub const DEFAULT_LABEL: Item<String> = Item::new("default_label");

/// Address for ACL contract used by this repo
pub const ACL_CONTRACT_ADDR: Item<Option<Addr>> = Item::new("acl_contract_addr");

/// Lookup tables going between managed contract "ID" and Addr
pub const ID_2_ADDR: Map<ContractID, Addr> = Map::new("id_2_addr");
pub const ADDR_2_ID: Map<Addr, ContractID> = Map::new("addr_2_id");

/// Total number of contracts in this repo
pub const COUNT: Item<u64> = Item::new("count");

/// Metadata stored for each contract in this repo
pub const METADATA: Map<Addr, ContractMetadata> = Map::new("metadata");

/// Built-in indices
pub const IX_CREATED_AT: U64IndexMap = Map::new("ix_created_at");
pub const IX_UPDATED_AT: U64IndexMap = Map::new("ix_updated_at");
pub const IX_CODE_ID: U64IndexMap = Map::new("ix_code_id");
pub const IX_REV: U64IndexMap = Map::new("ix_rev");

/// Custom index slots
// TODO: add seperate maps for None values
pub const IX_U64_0: U64IndexMap = Map::new("ix_u64_0");
pub const IX_U64_1: U64IndexMap = Map::new("ix_u64_1");
pub const IX_U64_2: U64IndexMap = Map::new("ix_u64_2");
pub const IX_U64_3: U64IndexMap = Map::new("ix_u64_3");
pub const IX_U64_4: U64IndexMap = Map::new("ix_u64_4");

pub const IX_TS_0: U64IndexMap = Map::new("ix_ts_u64_0");
pub const IX_TS_1: U64IndexMap = Map::new("ix_ts_u64_1");
pub const IX_TS_2: U64IndexMap = Map::new("ix_ts_u64_2");
pub const IX_TS_3: U64IndexMap = Map::new("ix_ts_u64_3");
pub const IX_TS_4: U64IndexMap = Map::new("ix_ts_u64_4");

pub const IX_STR_0: StrIndexMap = Map::new("ix_str_0");
pub const IX_STR_1: StrIndexMap = Map::new("ix_str_1");
pub const IX_STR_2: StrIndexMap = Map::new("ix_str_2");
pub const IX_STR_3: StrIndexMap = Map::new("ix_str_3");
pub const IX_STR_4: StrIndexMap = Map::new("ix_str_4");

/// Initialize contract state.
pub fn initialize(
  deps: DepsMut,
  _env: &Env,
  info: &MessageInfo,
  msg: &InstantiateMsg,
) -> Result<Response, ContractError> {
  CREATED_BY.save(deps.storage, &info.sender)?;
  ACL_CONTRACT_ADDR.save(deps.storage, &msg.acl_address)?;
  DEFAULT_LABEL.save(deps.storage, &msg.default_label)?;
  COUNT.save(deps.storage, &0)?;
  for code_id in msg.allowed_code_ids.iter() {
    ALLOWED_CODE_IDS.save(deps.storage, *code_id, &true)?;
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
pub fn get_and_increment_count(storage: &mut dyn Storage) -> Result<u64, ContractError> {
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

pub fn get_u64_index(index: u8) -> Result<U64IndexMap<'static>, ContractError> {
  match index {
    0 => Ok(IX_U64_0),
    1 => Ok(IX_U64_1),
    2 => Ok(IX_U64_2),
    3 => Ok(IX_U64_3),
    4 => Ok(IX_U64_4),
    _ => Err(ContractError::NotAuthorized {}),
  }
}

pub fn get_timestamp_index(index: u8) -> Result<U64IndexMap<'static>, ContractError> {
  match index {
    0 => Ok(IX_TS_0),
    1 => Ok(IX_TS_1),
    2 => Ok(IX_TS_2),
    3 => Ok(IX_TS_3),
    4 => Ok(IX_TS_4),
    _ => Err(ContractError::NotAuthorized {}),
  }
}

pub fn get_str_index(index: u8) -> Result<StrIndexMap<'static>, ContractError> {
  match index {
    0 => Ok(IX_STR_0),
    1 => Ok(IX_STR_1),
    2 => Ok(IX_STR_2),
    3 => Ok(IX_STR_3),
    4 => Ok(IX_STR_4),
    _ => Err(ContractError::NotAuthorized {}),
  }
}
