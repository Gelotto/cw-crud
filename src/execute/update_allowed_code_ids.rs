use crate::{
  error::ContractError,
  state::{is_allowed, ALLOWED_CODE_IDS},
};
use cosmwasm_std::{attr, DepsMut, Env, MessageInfo, Response};

pub fn update_allowed_code_ids(
  deps: DepsMut,
  _env: Env,
  info: MessageInfo,
  code_ids: Vec<u64>,
) -> Result<Response, ContractError> {
  if !is_allowed(
    deps.storage,
    &deps.querier,
    &info.sender,
    "update_allowed_code_ids",
  )? {
    return Err(ContractError::NotAuthorized {});
  }

  ALLOWED_CODE_IDS.clear(deps.storage);
  for code_id in code_ids.iter() {
    ALLOWED_CODE_IDS.save(deps.storage, *code_id, &true)?;
  }

  Ok(Response::new().add_attributes(vec![
    attr("action", "set_acl"),
    attr("code_ids", format!("{:?}", code_ids)),
  ]))
}
