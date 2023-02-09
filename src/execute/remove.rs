use crate::{
  error::ContractError,
  models::Slot,
  state::{
    decrement_index_size, get_bool_index, get_contract_id, get_text_index, get_timestamp_index,
    get_u64_index, is_allowed, ADDR_2_ID, BOOL_INDEX_METADATA, COUNT, ID_2_ADDR,
    ID_2_INDEXED_VALUES, IX_CODE_ID, IX_CREATED_AT, IX_HEIGHT, IX_REV, IX_UPDATED_AT, METADATA,
    TEXT_INDEX_METADATA, TS_INDEX_METADATA, UINT64_INDEX_METADATA,
  },
};
use cosmwasm_std::{attr, Addr, DepsMut, Env, MessageInfo, Response};

pub fn remove(
  deps: DepsMut,
  _env: Env,
  _info: MessageInfo,
  contract_addr: &Addr,
) -> Result<Response, ContractError> {
  if !is_allowed(deps.storage, &deps.querier, contract_addr, "remove")? {
    return Err(ContractError::NotAuthorized {});
  }

  let contract_id = get_contract_id(deps.storage, contract_addr)?;
  let meta = METADATA.load(deps.storage, contract_addr.clone())?;
  let prefixes = ID_2_INDEXED_VALUES.load(deps.storage, contract_id)?;

  ADDR_2_ID.remove(deps.storage, contract_addr.clone());
  ID_2_ADDR.remove(deps.storage, contract_id);

  for (i, some_value) in prefixes.uint64.iter().enumerate() {
    if let Some(value) = some_value {
      let slot = i as Slot;
      get_u64_index(slot)?.remove(deps.storage, (*value, contract_id));
      decrement_index_size(deps.storage, &UINT64_INDEX_METADATA, slot)?;
    }
  }
  for (i, some_value) in prefixes.text.iter().enumerate() {
    if let Some(value) = some_value {
      let slot = i as Slot;
      get_text_index(slot)?.remove(deps.storage, (value.clone(), contract_id));
      decrement_index_size(deps.storage, &TEXT_INDEX_METADATA, slot)?;
    }
  }
  for (i, some_value) in prefixes.timestamp.iter().enumerate() {
    if let Some(value) = some_value {
      let slot = i as Slot;
      get_timestamp_index(slot)?.remove(deps.storage, (*value, contract_id));
      decrement_index_size(deps.storage, &TS_INDEX_METADATA, slot)?;
    }
  }
  for (i, some_value) in prefixes.boolean.iter().enumerate() {
    if let Some(value) = some_value {
      let slot = i as Slot;
      get_bool_index(slot)?.remove(deps.storage, (*value, contract_id));
      decrement_index_size(deps.storage, &BOOL_INDEX_METADATA, slot)?;
    }
  }

  IX_CREATED_AT.remove(deps.storage, (meta.created_at.nanos(), contract_id));
  IX_UPDATED_AT.remove(deps.storage, (meta.updated_at.nanos(), contract_id));
  IX_REV.remove(deps.storage, (meta.rev, contract_id));
  IX_HEIGHT.remove(deps.storage, (meta.height, contract_id));
  IX_CODE_ID.remove(deps.storage, (meta.code_id, contract_id));

  COUNT.update(deps.storage, |count| -> Result<u64, ContractError> {
    Ok(count - 1)
  })?;

  Ok(Response::new().add_attributes(vec![
    attr("action", "remove"),
    attr("removed_contract_addr", contract_addr),
  ]))
}
