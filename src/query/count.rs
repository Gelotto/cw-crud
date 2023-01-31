use cosmwasm_std::Deps;

use crate::{error::ContractError, msg::CountResponse, state::COUNT};

/// Return total number of contracts in the repo.
pub fn count(deps: Deps) -> Result<CountResponse, ContractError> {
  Ok(CountResponse {
    count: COUNT.load(deps.storage)?,
  })
}
