use crate::{
  error::ContractError,
  models::{IndexMetadata, IndexSlotName, Slot, SLOT_COUNT},
  state::{is_allowed, IX_META_BOOL, IX_META_STRING, IX_META_TIMESTAMP, IX_META_U128, IX_META_U64},
};
use cosmwasm_std::{attr, DepsMut, Env, MessageInfo, Response, StdError, Storage};
use cw_storage_plus::Map;

pub fn rename_index(
  deps: DepsMut,
  _env: Env,
  info: MessageInfo,
  name: IndexSlotName,
) -> Result<Response, ContractError> {
  if !is_allowed(deps.storage, &deps.querier, &info.sender, "rename_index")? {
    return Err(ContractError::NotAuthorized {});
  }

  let (slot, (old_name, new_name)) = match name {
    IndexSlotName::Uint64 { slot, name } => {
      if slot >= SLOT_COUNT {
        return Err(ContractError::SlotOutOfBounds { slot });
      }
      (
        slot,
        update_index_name(deps.storage, &IX_META_U64, slot, &name)?,
      )
    },
    IndexSlotName::Timestamp { slot, name } => {
      if slot >= SLOT_COUNT {
        return Err(ContractError::SlotOutOfBounds { slot });
      }
      (
        slot,
        update_index_name(deps.storage, &IX_META_TIMESTAMP, slot, &name)?,
      )
    },
    IndexSlotName::Text { slot, name } => {
      if slot >= SLOT_COUNT {
        return Err(ContractError::SlotOutOfBounds { slot });
      }
      (
        slot,
        update_index_name(deps.storage, &IX_META_STRING, slot, &name)?,
      )
    },
    IndexSlotName::Boolean { slot, name } => {
      if slot >= SLOT_COUNT {
        return Err(ContractError::SlotOutOfBounds { slot });
      }
      (
        slot,
        update_index_name(deps.storage, &IX_META_BOOL, slot, &name)?,
      )
    },
    IndexSlotName::Uint128 { slot, name } => {
      if slot >= SLOT_COUNT {
        return Err(ContractError::SlotOutOfBounds { slot });
      }
      (
        slot,
        update_index_name(deps.storage, &IX_META_U128, slot, &name)?,
      )
    },
  };

  Ok(Response::new().add_attributes(vec![
    attr("action", "rename_index"),
    attr("old_name", old_name),
    attr("new_name", new_name),
    attr("slot", slot.to_string()),
  ]))
}

fn update_index_name<'a>(
  storage: &mut dyn Storage,
  map: &Map<'a, Slot, IndexMetadata>,
  slot: Slot,
  name: &Option<String>,
) -> Result<(String, String), ContractError> {
  let mut old_name: String = String::from("");
  let new_name: String = name.clone().unwrap_or(String::from(""));
  map.update(
    storage,
    slot,
    |some_meta: Option<IndexMetadata>| -> Result<IndexMetadata, ContractError> {
      if let Some(mut meta) = some_meta {
        old_name = meta.name.unwrap_or(String::from(""));
        meta.name = name.clone();
        Ok(meta)
      } else {
        // we should never reach this point because the index metadata
        // should have been initialized in instantiate.
        Err(
          StdError::GenericErr {
            msg: format!("index metadata not initialized"),
          }
          .into(),
        )
      }
    },
  )?;
  Ok((old_name, new_name))
}
