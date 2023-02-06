use cosmwasm_std::{Deps, Order, StdResult, Storage};
use cw_storage_plus::Map;

use crate::{
  error::ContractError,
  loader::RepositoryStateLoader,
  models::{IndexMetadata, IndexMetadataView, Slot},
  msg::SelectResponse,
  state::{
    ACL_CONTRACT_ADDR, ALLOWED_CODE_IDS, BOOL_INDEX_METADATA, COUNT, CREATED_BY, DEFAULT_CODE_ID,
    DEFAULT_LABEL, NUMBER_INDEX_METADATA, TEXT_INDEX_METADATA, TS_INDEX_METADATA,
  },
};

pub fn select(
  deps: Deps,
  fields: Option<Vec<String>>,
) -> Result<SelectResponse, ContractError> {
  let loader = RepositoryStateLoader::new(deps.storage, &fields);
  Ok(SelectResponse {
    count: loader.get("count", &COUNT)?,
    created_by: loader.get("created_by", &CREATED_BY)?,
    default_label: loader.get("default_label", &DEFAULT_LABEL)?,
    default_code_id: loader.get("default_code_id", &DEFAULT_CODE_ID)?,
    acl_address: loader.get("acl_address", &ACL_CONTRACT_ADDR)?,
    code_ids: loader.view("code_ids", || {
      Ok(Some(
        ALLOWED_CODE_IDS
          .keys(deps.storage, None, None, Order::Ascending)
          .map(|k| k.unwrap())
          .collect(),
      ))
    })?,
    indices: loader.view("indices", || {
      Ok(Some(IndexMetadataView {
        number: load_map_values(deps.storage, &NUMBER_INDEX_METADATA)?,
        text: load_map_values(deps.storage, &TEXT_INDEX_METADATA)?,
        boolean: load_map_values(deps.storage, &BOOL_INDEX_METADATA)?,
        timestamp: load_map_values(deps.storage, &TS_INDEX_METADATA)?,
      }))
    })?,
  })
}

fn load_map_values<'a>(
  storage: &dyn Storage,
  map: &Map<'a, Slot, IndexMetadata>,
) -> StdResult<Vec<IndexMetadata>> {
  Ok(
    map
      .range(storage, None, None, Order::Ascending)
      .map(|entry| {
        let (_, meta) = entry.unwrap();
        meta.clone()
      })
      .collect(),
  )
}
