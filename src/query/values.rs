use cosmwasm_std::{Addr, Deps};

use crate::{
  error::ContractError,
  msg::ValuesResponse,
  state::{get_contract_id, ID_2_INDEXED_VALUES},
};

pub fn values(
  deps: Deps,
  contract_addr: &Addr,
) -> Result<ValuesResponse, ContractError> {
  let contract_id = get_contract_id(deps.storage, contract_addr)?;
  let values = ID_2_INDEXED_VALUES.load(deps.storage, contract_id)?;
  Ok(ValuesResponse { values })
}
