use cosmwasm_std::{Deps, Order};

use crate::{
  error::ContractError,
  msg::SelectResponse,
  state::{ACL_CONTRACT_ADDR, ALLOWED_CODE_IDS, COUNT, CREATED_BY, DEFAULT_LABEL},
};

pub fn select(
  deps: Deps,
  _fields: Option<Vec<String>>,
) -> Result<SelectResponse, ContractError> {
  Ok(SelectResponse {
    created_by: CREATED_BY.load(deps.storage)?,
    default_label: DEFAULT_LABEL.load(deps.storage)?,
    acl_address: ACL_CONTRACT_ADDR.load(deps.storage)?,
    count: COUNT.load(deps.storage)?,
    allowed_code_ids: ALLOWED_CODE_IDS
      .keys(deps.storage, None, None, Order::Ascending)
      .map(|k| k.unwrap())
      .collect(),
  })
}
