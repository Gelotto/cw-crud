use crate::{
  error::ContractError,
  models::{AddressTag, IndexSlotValue, IndexedValues, InstantiationPreset, SLOT_COUNT},
  state::{
    get_bool_index, get_next_contract_id, get_text_index, get_timestamp_index, get_u128_index,
    get_u64_index, increment_index_size, is_allowed, ALLOWED_CODE_IDS, DEFAULT_CODE_ID,
    DEFAULT_LABEL, ID_2_INDEXED_VALUES, IX_CREATED_BY, IX_META_BOOL, IX_META_STRING,
    IX_META_TIMESTAMP, IX_META_U128, IX_META_U64, PRESETS, RELATIONSHIPS, TAGGED_CONTRACT_IDS,
  },
};
use cosmwasm_std::{
  attr, Addr, Binary, DepsMut, Env, MessageInfo, Response, Storage, SubMsg, WasmMsg,
};

/// Instantiate a managed contract. Extract its address in the reply entrypoint.
fn create(
  deps: DepsMut,
  env: Env,
  info: MessageInfo,
  maybe_code_id: Option<u64>,
  instantiate_msg: &Binary,
  maybe_admin: Option<Addr>,
  maybe_label: Option<String>,
  maybe_indices: Option<Vec<IndexSlotValue>>,
  maybe_save_as_preset_name: Option<String>,
  maybe_tags: Option<Vec<String>>,
  maybe_address_tags: Option<Vec<AddressTag>>,
) -> Result<Response, ContractError> {
  // the signer must be authorized to this method by the ACL
  if !is_allowed(deps.storage, &deps.querier, &info.sender, "create")? {
    return Err(ContractError::NotAuthorized {});
  }

  // use specified code ID for fall back on default
  let code_id = maybe_code_id.unwrap_or(DEFAULT_CODE_ID.load(deps.storage)?);

  // abort if code ID not whitelisted
  if !ALLOWED_CODE_IDS.has(deps.storage, code_id) {
    deps
      .api
      .debug(format!("code ID {} not allowed", code_id).as_str());
    return Err(ContractError::CodeIdNotAllowed {});
  }
  // we use the existing count AKA size of the collection as the ID
  // of the instantiate submsg as well as for its default label, if
  // necessary.
  let contract_id = get_next_contract_id(deps.storage)?;

  // store contract in association with the given tags
  for tag in maybe_tags.clone().unwrap_or_else(|| vec![]).iter() {
    TAGGED_CONTRACT_IDS.save(deps.storage, (tag.clone(), contract_id), &true)?;
  }

  // store tagged addresses
  for addr_tag in maybe_address_tags.unwrap_or(vec![]).iter() {
    let key = (addr_tag.address.clone(), addr_tag.tag.clone(), contract_id);
    RELATIONSHIPS.save(deps.storage, key, &true)?;
  }

  IX_CREATED_BY.save(deps.storage, (info.sender.clone(), contract_id), &true)?;

  // we use "keys" to keep track of which custom index keys are associated
  // with the new contract ID because we'll need this for the sake up updating
  // and removing contracts from the repo.
  let mut keys = IndexedValues::new();

  // initialize custom indices
  if let Some(indices) = &maybe_indices {
    for params in indices.iter() {
      match params.clone() {
        IndexSlotValue::Uint64 { slot, value } => {
          if slot >= SLOT_COUNT {
            return Err(ContractError::SlotOutOfBounds { slot });
          }
          increment_index_size(deps.storage, &IX_META_U64, slot)?;
          get_u64_index(slot)?.save(deps.storage, (value, contract_id), &true)?;
          keys.uint64[slot as usize] = Some(value);
        },
        IndexSlotValue::Timestamp { slot, value } => {
          if slot >= SLOT_COUNT {
            return Err(ContractError::SlotOutOfBounds { slot });
          }
          increment_index_size(deps.storage, &IX_META_TIMESTAMP, slot)?;
          get_timestamp_index(slot)?.save(deps.storage, (value.nanos(), contract_id), &true)?;
          keys.timestamp[slot as usize] = Some(value.nanos());
        },
        IndexSlotValue::Text { slot, value } => {
          if slot >= SLOT_COUNT {
            return Err(ContractError::SlotOutOfBounds { slot });
          }
          increment_index_size(deps.storage, &IX_META_STRING, slot)?;
          get_text_index(slot)?.save(deps.storage, (value.clone(), contract_id), &true)?;
          keys.text[slot as usize] = Some(value.clone());
        },
        IndexSlotValue::Boolean { slot, value } => {
          if slot >= SLOT_COUNT {
            return Err(ContractError::SlotOutOfBounds { slot });
          }
          let u8_bool = if value { 1 } else { 0 };
          increment_index_size(deps.storage, &IX_META_BOOL, slot)?;
          get_bool_index(slot)?.save(deps.storage, (u8_bool, contract_id), &true)?;
          keys.boolean[slot as usize] = Some(u8_bool);
        },
        IndexSlotValue::Uint128 { slot, value } => {
          if slot >= SLOT_COUNT {
            return Err(ContractError::SlotOutOfBounds { slot });
          }
          increment_index_size(deps.storage, &IX_META_U128, slot)?;
          get_u128_index(slot)?.save(deps.storage, (value, contract_id), &true)?;
          keys.uint128[slot as usize] = Some(value);
        },
      }
    }
  }

  ID_2_INDEXED_VALUES.save(deps.storage, contract_id, &keys)?;

  let computed_label = build_label(deps.storage, maybe_label, contract_id)?;
  let computed_admin = maybe_admin
    .clone()
    .and_then(|addr| Some(addr.to_string()))
    .or(Some(env.contract.address.into()));

  // create instantiation submsg. The instantiated contract should store the
  // sender address (of this repository contract) for it to use when calling update or
  // other methods defined by the Repository.
  let wasm_instantiate_msg = WasmMsg::Instantiate {
    code_id,
    msg: instantiate_msg.clone(),
    funds: info.funds,
    label: computed_label.clone(),
    admin: computed_admin.clone(),
  };

  if let Some(preset_name) = &maybe_save_as_preset_name {
    PRESETS.update(
      deps.storage,
      (info.sender.clone(), preset_name.clone()),
      |maybe_preset| -> Result<InstantiationPreset, ContractError> {
        if maybe_preset.is_none() {
          Ok(InstantiationPreset {
            code_id: Some(code_id),
            msg: instantiate_msg.clone(),
            tags: maybe_tags.clone(),
            indices: maybe_indices.clone(),
            label: Some(computed_label.clone()),
            admin: computed_admin
              .clone()
              .and_then(|s| Some(Addr::unchecked(s))),
          })
        } else {
          Err(ContractError::PresetExists {})
        }
      },
    )?;
  }

  Ok(
    Response::new()
      .add_attributes(vec![
        attr("action", "create"),
        attr("code_id", code_id.to_string()),
        attr("admin", computed_admin.clone().unwrap()),
        attr("label", computed_label.clone()),
      ])
      .add_submessage(SubMsg::reply_always(wasm_instantiate_msg, contract_id)),
  )
}

pub fn create_from_preset(
  deps: DepsMut,
  env: Env,
  info: MessageInfo,
  maybe_code_id: Option<u64>,
  maybe_instantiate_msg: Option<Binary>,
  maybe_admin: Option<Addr>,
  maybe_label: Option<String>,
  maybe_indices: Option<Vec<IndexSlotValue>>,
  maybe_preset_name: Option<String>,
  maybe_save_as_preset_name: Option<String>,
  maybe_tags: Option<Vec<String>>,
  maybe_address_tags: Option<Vec<AddressTag>>,
) -> Result<Response, ContractError> {
  if let Some(preset_name) = maybe_preset_name {
    let preset = PRESETS.load(deps.storage, (info.sender.clone(), preset_name.clone()))?;
    create(
      deps,
      env,
      info,
      maybe_code_id.or(preset.code_id),
      &maybe_instantiate_msg.unwrap_or(preset.msg),
      maybe_admin.or(preset.admin),
      maybe_label.or(preset.label),
      maybe_indices.or(preset.indices),
      maybe_save_as_preset_name,
      maybe_tags,
      maybe_address_tags,
    )
  } else {
    create(
      deps,
      env,
      info,
      maybe_code_id,
      &maybe_instantiate_msg.unwrap_or(Binary::from_base64("e30=")?),
      maybe_admin,
      maybe_label,
      maybe_indices,
      maybe_save_as_preset_name,
      maybe_tags,
      maybe_address_tags,
    )
  }
}

/// Build or use default label for instantiated contract.
fn build_label(
  storage: &dyn Storage,
  custom_label: Option<String>,
  n: u64,
) -> Result<String, ContractError> {
  if let Some(label) = custom_label {
    Ok(label)
  } else {
    let some_default_label = DEFAULT_LABEL.load(storage)?;
    if let Some(default_label) = some_default_label {
      Ok(format!("{}-{:?}", default_label, n))
    } else {
      Err(ContractError::LabelRequired {})
    }
  }
}
