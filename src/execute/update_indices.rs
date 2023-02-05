use crate::{
  error::ContractError,
  models::{ContractID, IndexMetadata, IndexPrefix, IndexUpdate, Slot},
  state::{
    get_bool_index, get_contract_id, get_number_index, get_text_index, get_timestamp_index,
    BOOL_INDEX_METADATA, IX_REV, METADATA, NUMBER_INDEX_METADATA, TEXT_INDEX_METADATA,
    TS_INDEX_METADATA,
  },
  state::{owns_contract, IX_UPDATED_AT},
};
use cosmwasm_std::{attr, DepsMut, Env, MessageInfo, Response, Storage, Timestamp};
use cw_storage_plus::Map;

pub fn update_indices(
  deps: DepsMut,
  env: Env,
  info: MessageInfo,
  index_updates: Option<Vec<IndexUpdate>>,
) -> Result<Response, ContractError> {
  let contract_addr = &info.sender;

  deps
    .api
    .debug(format!("executing repository update for: {}", info.sender).as_str());

  if !owns_contract(deps.storage, contract_addr) {
    return Err(ContractError::NotAuthorized {});
  }

  let id = get_contract_id(deps.storage, contract_addr)?;
  let mut meta = METADATA.load(deps.storage, contract_addr.clone())?;

  // update updated_at index
  if IX_UPDATED_AT.has(deps.storage, (meta.updated_at.nanos(), id)) {
    IX_UPDATED_AT.remove(deps.storage, (meta.updated_at.nanos(), id));
    IX_UPDATED_AT.save(deps.storage, (env.block.time.nanos(), id), &true)?;
  } else {
    return Err(ContractError::NotInIndex {
      msg: format!("old value not in updated_at index"),
    });
  }

  // update rev index
  if IX_REV.has(deps.storage, (meta.rev, id)) {
    IX_REV.remove(deps.storage, (meta.rev, id));
    IX_REV.save(deps.storage, (meta.rev + 1, id), &true)?;
  } else {
    return Err(ContractError::NotInIndex {
      msg: format!("old value not in rev index"),
    });
  }

  // update managed contract metadata
  meta.updated_at = env.block.time;
  meta.rev += 1;

  METADATA.save(deps.storage, contract_addr.clone(), &meta)?;

  // update other indices
  if let Some(updates) = index_updates {
    for u in updates.iter() {
      match &u.values {
        crate::models::IndexUpdateValues::Number(old_val, new_val) => {
          update_number_index(deps.storage, &env, u.slot, *old_val, *new_val, id)?;
        },
        crate::models::IndexUpdateValues::Text(old_val, new_val) => {
          update_text_index(deps.storage, &env, u.slot, old_val, new_val, id)?;
        },
        crate::models::IndexUpdateValues::Timestamp(old_val, new_val) => {
          update_timestamp_index(deps.storage, &env, u.slot, old_val, new_val, id)?;
        },
        crate::models::IndexUpdateValues::Boolean(old_val, new_val) => {
          update_bool_index(deps.storage, &env, u.slot, old_val, new_val, id)?;
        },
      }
    }
  }

  Ok(Response::new().add_attributes(vec![attr("action", "update")]))
}

fn update_index_metadata<'a>(
  storage: &mut dyn Storage,
  env: &Env,
  map: &Map<'a, Slot, IndexMetadata>,
  slot: Slot,
  contract_id: ContractID,
  prefix: IndexPrefix,
) -> Result<IndexMetadata, ContractError> {
  Ok(map.update(
    storage,
    slot,
    |some_meta| -> Result<IndexMetadata, ContractError> {
      if let Some(mut meta) = some_meta {
        meta.updated_at = Some(env.block.time);
        meta.updated_key = Some((prefix, contract_id));
        Ok(meta)
      } else {
        Err(ContractError::InvalidIndexSlot {})
      }
    },
  )?)
}

fn update_number_index(
  storage: &mut dyn Storage,
  env: &Env,
  slot: Slot,
  old_val: u64,
  new_val: u64,
  id: ContractID,
) -> Result<u64, ContractError> {
  let map = get_number_index(slot)?;
  if map.has(storage, (old_val.clone(), id)) {
    map.remove(storage, (old_val.clone(), id));
    map.save(storage, (new_val.clone(), id), &true)?;
    update_index_metadata(
      storage,
      env,
      &NUMBER_INDEX_METADATA,
      slot,
      id,
      IndexPrefix::Number(new_val),
    )?;
    Ok(new_val)
  } else {
    Err(ContractError::NotInIndex {
      msg: format!("old value not in u64 index slot {}", slot),
    })
  }
}

fn update_text_index(
  storage: &mut dyn Storage,
  env: &Env,
  slot: Slot,
  old_val: &String,
  new_val: &String,
  id: ContractID,
) -> Result<String, ContractError> {
  let map = get_text_index(slot)?;
  if map.has(storage, (old_val.clone(), id)) {
    map.remove(storage, (old_val.clone(), id));
    map.save(storage, (new_val.clone(), id), &true)?;
    update_index_metadata(
      storage,
      env,
      &TEXT_INDEX_METADATA,
      slot,
      id,
      IndexPrefix::Text(new_val.clone()),
    )?;
    Ok(new_val.clone())
  } else {
    Err(ContractError::NotInIndex {
      msg: format!("old value not in String index slot {}", slot),
    })
  }
}

fn update_timestamp_index(
  storage: &mut dyn Storage,
  env: &Env,
  slot: Slot,
  old_val: &Timestamp,
  new_val: &Timestamp,
  id: ContractID,
) -> Result<Timestamp, ContractError> {
  let map = get_timestamp_index(slot)?;
  if map.has(storage, (old_val.nanos(), id)) {
    map.remove(storage, (old_val.nanos(), id));
    map.save(storage, (new_val.nanos(), id), &true)?;
    update_index_metadata(
      storage,
      env,
      &TS_INDEX_METADATA,
      slot,
      id,
      IndexPrefix::Timestamp(new_val.nanos()),
    )?;
    Ok(new_val.clone())
  } else {
    Err(ContractError::NotInIndex {
      msg: format!("old value not in timestamp index slot {}", slot),
    })
  }
}

fn update_bool_index(
  storage: &mut dyn Storage,
  env: &Env,
  slot: u8,
  old_val: &bool,
  new_val: &bool,
  id: ContractID,
) -> Result<bool, ContractError> {
  let map = get_bool_index(slot)?;
  let old_u8_bool = if *old_val { 1 } else { 0 };
  let new_u8_bool = if *new_val { 1 } else { 0 };
  if map.has(storage, (old_u8_bool.clone(), id)) {
    map.remove(storage, (old_u8_bool, id));
    map.save(storage, (new_u8_bool, id), &true)?;
    update_index_metadata(
      storage,
      env,
      &BOOL_INDEX_METADATA,
      slot,
      id,
      IndexPrefix::Boolean(new_u8_bool),
    )?;
    Ok(new_val.clone())
  } else {
    Err(ContractError::NotInIndex {
      msg: format!("old value not in bool index slot {}", slot),
    })
  }
}
