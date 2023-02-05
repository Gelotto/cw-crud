use crate::{
  error::ContractError,
  models::IndexSlotValue,
  state::{
    get_bool_index, get_next_contract_id, get_number_index, get_text_index, get_timestamp_index,
    is_allowed, update_index_metadata_count, ALLOWED_CODE_IDS, BOOL_INDEX_METADATA,
    DEFAULT_CODE_ID, DEFAULT_LABEL, IX_CREATED_BY, NUMBER_INDEX_METADATA, TEXT_INDEX_METADATA,
    TS_INDEX_METADATA,
  },
};
use cosmwasm_std::{
  attr, Addr, Binary, DepsMut, Env, MessageInfo, Response, Storage, SubMsg, WasmMsg,
};

/// Instantiate a managed contract. Extract its address in the reply entrypoint.
pub fn create(
  deps: DepsMut,
  _env: Env,
  info: MessageInfo,
  code_id_override: Option<u64>,
  instantiate_msg: &Binary,
  admin: Option<Addr>,
  label: Option<String>,
  indices: Option<Vec<IndexSlotValue>>,
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

  // initialize custom indices
  for params in indices.unwrap_or(vec![]).clone().iter() {
    match params {
      IndexSlotValue::Number { slot, value } => {
        update_index_metadata_count(deps.storage, &NUMBER_INDEX_METADATA, *slot)?;
        get_number_index(*slot)?.save(deps.storage, (*value, contract_id), &true)?;
      },
      IndexSlotValue::Timestamp { slot, value } => {
        update_index_metadata_count(deps.storage, &TS_INDEX_METADATA, *slot)?;
        get_timestamp_index(*slot)?.save(deps.storage, (value.nanos(), contract_id), &true)?;
      },
      IndexSlotValue::Text { slot, value } => {
        update_index_metadata_count(deps.storage, &TEXT_INDEX_METADATA, *slot)?;
        get_text_index(*slot)?.save(deps.storage, (value.clone(), contract_id), &true)?;
      },
      IndexSlotValue::Boolean { slot, value } => {
        update_index_metadata_count(deps.storage, &BOOL_INDEX_METADATA, *slot)?;
        get_bool_index(*slot)?.save(
          deps.storage,
          (if *value { 1 } else { 0 }, contract_id),
          &true,
        )?;
      },
    }
  }

  // create instantiation submsg. The instantiated contract should store the
  // sender address (of this repository contract) for it to use when calling update or
  // other methods defined by the Repository.
  let wasm_instantiate_msg = WasmMsg::Instantiate {
    code_id,
    admin: admin.and_then(|addr| Some(addr.to_string())).or(None),
    msg: instantiate_msg.clone(),
    funds: info.funds,
    label: build_label(deps.storage, label, contract_id)?,
  };

  Ok(
    Response::new()
      .add_attributes(vec![attr("action", "create"), attr("code_id", info.sender)])
      .add_submessage(SubMsg::reply_always(wasm_instantiate_msg, contract_id)),
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
