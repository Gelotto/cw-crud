use crate::{
  error::ContractError,
  models::IndexInitializationParams,
  state::{
    get_and_increment_count, get_str_index, get_timestamp_index, get_u64_index, is_allowed,
    ALLOWED_CODE_IDS, DEFAULT_LABEL,
  },
};
use cosmwasm_std::{
  attr, Addr, Binary, Coin, DepsMut, Env, MessageInfo, Response, Storage, SubMsg, WasmMsg,
};

/// Instantiate a managed contract. Extract its address in the reply entrypoint.
pub fn create(
  deps: DepsMut,
  _env: Env,
  info: MessageInfo,
  code_id: u64,
  instantiate_msg: &Binary,
  admin: Option<Addr>,
  funds: Option<Vec<Coin>>,
  label: Option<String>,
  indices: Option<Vec<IndexInitializationParams>>,
) -> Result<Response, ContractError> {
  // the signer must be authorized to this method by the ACL
  if !is_allowed(deps.storage, &deps.querier, &info.sender, "create")? {
    return Err(ContractError::NotAuthorized {});
  }

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
  let id = get_and_increment_count(deps.storage)?;

  // initialize custom indices
  for params in indices.unwrap_or(vec![]).clone().iter() {
    match params {
      IndexInitializationParams::Numeric { idx, value } => {
        let map = get_u64_index(*idx)?;
        map.save(deps.storage, (*value, id), &true)?;
      },
      IndexInitializationParams::Timestamp { idx, value } => {
        let map = get_timestamp_index(*idx)?;
        map.save(deps.storage, (value.nanos(), id), &true)?;
      },
      IndexInitializationParams::Text { idx, value } => {
        let map = get_str_index(*idx)?;
        map.save(deps.storage, (value.clone(), id), &true)?;
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
    funds: funds.unwrap_or(vec![]),
    label: build_label(deps.storage, label, id)?,
  };

  Ok(
    Response::new()
      .add_attributes(vec![attr("action", "create"), attr("code_id", info.sender)])
      .add_submessage(SubMsg::reply_always(wasm_instantiate_msg, id)),
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
    let base = DEFAULT_LABEL.load(storage)?;
    Ok(format!("{}-{:?}", base, n))
  }
}
