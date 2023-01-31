use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContractError {
  #[error("{0}")]
  Std(#[from] StdError),

  #[error("NotAuthorized")]
  NotAuthorized {},

  #[error("CreateFailed")]
  CreateFailed {},

  #[error("CodeIdNotAllowed")]
  CodeIdNotAllowed {},

  #[error("AclAlreadyEnabled")]
  AclAlreadyEnabled {},

  #[error("AlreadyExists")]
  AlreadyExists {},

  #[error("NotInView")]
  NotInView {},
}
