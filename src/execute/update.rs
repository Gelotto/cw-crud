use crate::{
  error::ContractError,
  models::{ContractID, IndexUpdate},
  state::{get_contract_id, get_str_index, get_u64_index, IX_REV, METADATA},
  state::{owns_contract, IX_UPDATED_AT},
};
use cosmwasm_std::{attr, DepsMut, Env, MessageInfo, Response, Storage};

pub fn update(
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

  // update built-in indices
  IX_UPDATED_AT.remove(deps.storage, (meta.updated_at.nanos(), id));
  IX_UPDATED_AT.save(deps.storage, (env.block.time.nanos(), id), &true)?;

  IX_REV.remove(deps.storage, (meta.rev, id));
  IX_REV.save(deps.storage, (meta.rev + 1, id), &true)?;

  // update managed contract metadata
  meta.updated_at = env.block.time;
  meta.rev += 1;

  METADATA.save(deps.storage, contract_addr.clone(), &meta)?;

  // update other indices
  if let Some(view_updates) = index_updates {
    for vu in view_updates.iter() {
      match &vu.values {
        crate::models::IndexUpdateValues::Numeric(old_val, new_val) => {
          update_numeric_index(deps.storage, vu.index, *old_val, *new_val, id)?;
        },
        crate::models::IndexUpdateValues::Text(old_val, new_val) => {
          update_text_index(deps.storage, vu.index, old_val, new_val, id)?;
        },
      }
    }
  }

  Ok(Response::new().add_attributes(vec![attr("action", "update")]))
}

fn update_numeric_index(
  storage: &mut dyn Storage,
  ix_index: u8,
  old_val: u64,
  new_val: u64,
  id: ContractID,
) -> Result<u64, ContractError> {
  let map = get_u64_index(ix_index)?;
  map.remove(storage, (old_val.clone(), id));
  map.save(storage, (new_val.clone(), id), &true)?;
  return Ok(new_val);
}

fn update_text_index(
  storage: &mut dyn Storage,
  ix_index: u8,
  old_val: &String,
  new_val: &String,
  id: ContractID,
) -> Result<String, ContractError> {
  let map = get_str_index(ix_index)?;
  map.remove(storage, (old_val.clone(), id));
  map.save(storage, (new_val.clone(), id), &true)?;
  return Ok(new_val.clone());
}
