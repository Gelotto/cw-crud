use cosmwasm_std::StdError;
use thiserror::Error;

use crate::models::Slot;

#[derive(Debug, Error)]
pub enum ContractError {
  #[error("{0}")]
  Std(#[from] StdError),

  #[error("NotAuthorized")]
  NotAuthorized {},

  #[error("CreateFailed")]
  CreateFailed {},

  #[error("InvalidLabel")]
  InvalidLabel {},

  #[error("NoAllowedCodeIds")]
  NoAllowedCodeIds {},

  #[error("CodeIdNotAllowed")]
  CodeIdNotAllowed {},

  #[error("NotFound")]
  NotFound {},

  #[error("AclAlreadyEnabled")]
  AclAlreadyEnabled {},

  #[error("PresetExists")]
  PresetExists {},

  #[error("AlreadyExists")]
  AlreadyExists {},

  #[error("NotInIndex")]
  NotInIndex { msg: String },

  #[error("QueryStateError")]
  QueryStateError { msg: String },

  #[error("ValidationError")]
  ValidationError { msg: String },

  #[error("LabelRequired")]
  LabelRequired {},

  #[error("InvalidIndexSlot")]
  InvalidIndexSlot {},

  #[error("SlotOutOfBounds")]
  SlotOutOfBounds { slot: Slot },
}
