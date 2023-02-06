use crate::{
  error::ContractError,
  state::{is_allowed, ACL_CONTRACT_ADDR},
};
use cosmwasm_std::{attr, Addr, DepsMut, Env, MessageInfo, Response};

pub fn set_acl(
  deps: DepsMut,
  _env: Env,
  info: MessageInfo,
  acl_contract_addr: &Addr,
) -> Result<Response, ContractError> {
  if !is_allowed(deps.storage, &deps.querier, &info.sender, "set_acl")? {
    return Err(ContractError::NotAuthorized {});
  }

  ACL_CONTRACT_ADDR.save(deps.storage, &Some(acl_contract_addr.clone()))?;

  Ok(Response::new().add_attributes(vec![
    attr("action", "set_acl"),
    attr("acl_contract_addr", acl_contract_addr.to_string()),
  ]))
}
