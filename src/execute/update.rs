use crate::{
  error::ContractError,
  models::{ContractID, IndexMetadata, IndexPrefix, IndexSlotValue, Slot, SLOT_COUNT},
  state::{
    get_bool_index, get_contract_id, get_text_index, get_timestamp_index, get_u128_index,
    get_u64_index, increment_index_size, ID_2_INDEXED_VALUES, IX_META_BOOL, IX_META_STRING,
    IX_META_TIMESTAMP, IX_META_U128, IX_META_U64, IX_REV, METADATA,
  },
  state::{owns_contract, IX_UPDATED_AT},
};
use cosmwasm_std::{attr, DepsMut, Env, MessageInfo, Response, Storage, Timestamp};
use cw_storage_plus::Map;

pub fn update(
  deps: DepsMut,
  env: Env,
  info: MessageInfo,
  index_updates: Option<Vec<IndexSlotValue>>,
) -> Result<Response, ContractError> {
  let contract_addr = &info.sender;

  deps
    .api
    .debug(format!("executing repository update for: {}", info.sender).as_str());

  if !owns_contract(deps.storage, contract_addr) {
    return Err(ContractError::NotAuthorized {});
  }

  let contract_id = get_contract_id(deps.storage, contract_addr)?;
  let mut meta = METADATA.load(deps.storage, contract_addr.clone())?;

  // update updated_at index
  if IX_UPDATED_AT.has(deps.storage, (meta.updated_at.nanos(), contract_id)) {
    IX_UPDATED_AT.remove(deps.storage, (meta.updated_at.nanos(), contract_id));
    IX_UPDATED_AT.save(deps.storage, (env.block.time.nanos(), contract_id), &true)?;
  } else {
    return Err(ContractError::NotInIndex {
      msg: format!("old value not in updated_at index"),
    });
  }

  // update rev index
  if IX_REV.has(deps.storage, (meta.rev, contract_id)) {
    IX_REV.remove(deps.storage, (meta.rev, contract_id));
    IX_REV.save(deps.storage, (meta.rev + 1, contract_id), &true)?;
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
    let mut ix_keys = ID_2_INDEXED_VALUES.load(deps.storage, contract_id)?;

    for u in updates.iter() {
      match u.clone() {
        IndexSlotValue::Uint64 { slot, value } => {
          if slot >= SLOT_COUNT {
            return Err(ContractError::SlotOutOfBounds { slot });
          }
          let old_val = ix_keys.uint64[slot as usize];
          update_u64_index(deps.storage, &env, slot, old_val, value, contract_id)?;
          ix_keys.uint64[slot as usize] = Some(value);
        },
        IndexSlotValue::Text { slot, value } => {
          if slot >= SLOT_COUNT {
            return Err(ContractError::SlotOutOfBounds { slot });
          }
          let old_val = ix_keys.text[slot as usize].clone();
          update_text_index(deps.storage, &env, slot, old_val, &value, contract_id)?;
          ix_keys.text[slot as usize] = Some(value.clone());
        },
        IndexSlotValue::Timestamp { slot, value } => {
          if slot >= SLOT_COUNT {
            return Err(ContractError::SlotOutOfBounds { slot });
          }
          let old_val = ix_keys.timestamp[slot as usize];
          update_timestamp_index(deps.storage, &env, slot, old_val, &value, contract_id)?;
          ix_keys.timestamp[slot as usize] = Some(value.nanos());
        },
        IndexSlotValue::Boolean { slot, value } => {
          if slot >= SLOT_COUNT {
            return Err(ContractError::SlotOutOfBounds { slot });
          }
          let old_val = ix_keys.boolean[slot as usize];
          update_bool_index(deps.storage, &env, slot, old_val, &value, contract_id)?;
          ix_keys.timestamp[slot as usize] = Some(if value { 1 } else { 0 });
        },
        IndexSlotValue::Uint128 { slot, value } => {
          if slot >= SLOT_COUNT {
            return Err(ContractError::SlotOutOfBounds { slot });
          }
          let old_val = ix_keys.uint128[slot as usize];
          update_u128_index(deps.storage, &env, slot, old_val, value, contract_id)?;
          ix_keys.uint128[slot as usize] = Some(value);
        },
      }
    }

    ID_2_INDEXED_VALUES.save(deps.storage, contract_id, &ix_keys)?;
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

fn update_u64_index(
  storage: &mut dyn Storage,
  env: &Env,
  slot: Slot,
  some_old_val: Option<u64>,
  new_val: u64,
  id: ContractID,
) -> Result<u64, ContractError> {
  let map = get_u64_index(slot)?;

  if let Some(old_val) = some_old_val {
    map.remove(storage, (old_val.clone(), id));
  } else {
    increment_index_size(storage, &IX_META_U64, slot)?;
  }

  map.save(storage, (new_val.clone(), id), &true)?;

  update_index_metadata(
    storage,
    env,
    &IX_META_U64,
    slot,
    id,
    IndexPrefix::Uint64(new_val),
  )?;

  Ok(new_val)
}

fn update_u128_index(
  storage: &mut dyn Storage,
  env: &Env,
  slot: Slot,
  some_old_val: Option<u128>,
  new_val: u128,
  id: ContractID,
) -> Result<u128, ContractError> {
  let map = get_u128_index(slot)?;

  if let Some(old_val) = some_old_val {
    map.remove(storage, (old_val.clone(), id));
  } else {
    increment_index_size(storage, &IX_META_U128, slot)?;
  }

  map.save(storage, (new_val.clone(), id), &true)?;

  update_index_metadata(
    storage,
    env,
    &IX_META_U128,
    slot,
    id,
    IndexPrefix::Uint128(new_val),
  )?;

  Ok(new_val)
}

fn update_text_index(
  storage: &mut dyn Storage,
  env: &Env,
  slot: Slot,
  some_old_val: Option<String>,
  new_val: &String,
  id: ContractID,
) -> Result<String, ContractError> {
  let map = get_text_index(slot)?;

  if let Some(old_val) = some_old_val {
    map.remove(storage, (old_val.clone(), id));
  } else {
    increment_index_size(storage, &IX_META_STRING, slot)?;
  }

  map.save(storage, (new_val.clone(), id), &true)?;

  update_index_metadata(
    storage,
    env,
    &IX_META_STRING,
    slot,
    id,
    IndexPrefix::Text(new_val.clone()),
  )?;

  Ok(new_val.clone())
}

fn update_timestamp_index(
  storage: &mut dyn Storage,
  env: &Env,
  slot: Slot,
  some_old_val: Option<u64>,
  new_val: &Timestamp,
  id: ContractID,
) -> Result<Timestamp, ContractError> {
  let map = get_timestamp_index(slot)?;

  if let Some(old_val) = some_old_val {
    map.remove(storage, (old_val, id));
  } else {
    increment_index_size(storage, &IX_META_TIMESTAMP, slot)?;
  }

  map.save(storage, (new_val.nanos(), id), &true)?;

  update_index_metadata(
    storage,
    env,
    &IX_META_TIMESTAMP,
    slot,
    id,
    IndexPrefix::Timestamp(new_val.nanos()),
  )?;
  Ok(new_val.clone())
}

fn update_bool_index(
  storage: &mut dyn Storage,
  env: &Env,
  slot: u8,
  some_old_val: Option<u8>,
  new_val: &bool,
  id: ContractID,
) -> Result<bool, ContractError> {
  let map = get_bool_index(slot)?;

  if let Some(old_val) = some_old_val {
    map.remove(storage, (old_val, id));
  } else {
    increment_index_size(storage, &IX_META_STRING, slot)?;
  }

  let new_u8_bool = if *new_val { 1 } else { 0 };

  map.save(storage, (new_u8_bool, id), &true)?;
  update_index_metadata(
    storage,
    env,
    &IX_META_BOOL,
    slot,
    id,
    IndexPrefix::Boolean(new_u8_bool),
  )?;

  Ok(new_val.clone())
}
