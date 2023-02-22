use crate::{error::ContractError, state::INSTANTIATION_PRESETS};
use cosmwasm_std::{attr, DepsMut, Env, MessageInfo, Response};

pub fn remove_preset(
  deps: DepsMut,
  _env: Env,
  info: MessageInfo,
  preset_name: &String,
) -> Result<Response, ContractError> {
  let key = (info.sender.clone(), preset_name.clone());
  if !INSTANTIATION_PRESETS.has(deps.storage, key.clone()) {
    return Err(ContractError::NotAuthorized {});
  }

  INSTANTIATION_PRESETS.remove(deps.storage, key.clone());

  Ok(Response::new().add_attributes(vec![
    attr("action", "remove_preset"),
    attr("preset", preset_name),
  ]))
}
