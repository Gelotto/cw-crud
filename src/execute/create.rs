use crate::{
  error::ContractError,
  models::{IndexSlotValue, IndexedValues, InstantiationPreset, SLOT_COUNT},
  state::{
    get_bool_index, get_next_contract_id, get_text_index, get_timestamp_index, get_u128_index,
    get_u64_index, increment_index_size, is_allowed, ALLOWED_CODE_IDS, DEFAULT_CODE_ID,
    DEFAULT_LABEL, ID_2_INDEXED_VALUES, INSTANTIATION_PRESETS, IX_CREATED_BY, IX_META_BOOL,
    IX_META_STRING, IX_META_TIMESTAMP, IX_META_U128, IX_META_U64,
  },
};
use cosmwasm_std::{
  attr, Addr, Binary, DepsMut, Env, MessageInfo, Response, Storage, SubMsg, WasmMsg,
};

/// Instantiate a managed contract. Extract its address in the reply entrypoint.
pub fn create(
  deps: DepsMut,
  env: Env,
  info: MessageInfo,
  code_id_override: Option<u64>,
  instantiate_msg: &Binary,
  admin: Option<Addr>,
  label: Option<String>,
  indices: Option<Vec<IndexSlotValue>>,
  maybe_preset_name: Option<String>,
) -> Result<Response, ContractError> {
  // the signer must be authorized to this method by the ACL
  if !is_allowed(deps.storage, &deps.querier, &info.sender, "create")? {
    return Err(ContractError::NotAuthorized {});
  }

  // use specified code ID for fall back on default
  let code_id = code_id_override.unwrap_or(DEFAULT_CODE_ID.load(deps.storage)?);

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

  IX_CREATED_BY.save(deps.storage, (info.sender.clone(), contract_id), &true)?;

  // we use "keys" to keep track of which custom index keys are associated
  // with the new contract ID because we'll need this for the sake up updating
  // and removing contracts from the repo.
  let mut keys = IndexedValues::new();

  // initialize custom indices
  if let Some(indices) = &indices {
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

  let computed_label = build_label(deps.storage, label, contract_id)?;
  let computed_admin = admin
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

  if let Some(preset_name) = &maybe_preset_name {
    INSTANTIATION_PRESETS.update(
      deps.storage,
      (admin.unwrap_or(info.sender).clone(), preset_name.clone()),
      |maybe_preset| -> Result<InstantiationPreset, ContractError> {
        if maybe_preset.is_none() {
          Ok(InstantiationPreset {
            code_id: Some(code_id),
            msg: instantiate_msg.clone(),
            indices: indices.clone(),
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
  code_id_override: Option<u64>,
  instantiate_msg: Option<Binary>,
  admin: Option<Addr>,
  label: Option<String>,
  indices: Option<Vec<IndexSlotValue>>,
  owner_addr: Addr,
  preset_name: String,
) -> Result<Response, ContractError> {
  let preset =
    INSTANTIATION_PRESETS.load(deps.storage, (owner_addr.clone(), preset_name.clone()))?;

  create(
    deps,
    env,
    info,
    code_id_override.or(preset.code_id),
    &instantiate_msg.unwrap_or(preset.msg),
    admin.or(preset.admin),
    label.or(preset.label),
    indices.or(preset.indices),
    None,
  )
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
