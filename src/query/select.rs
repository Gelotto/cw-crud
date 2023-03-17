use cosmwasm_std::{Addr, Deps, Order, StdResult, Storage};
use cw_storage_plus::Map;

use crate::{
  error::ContractError,
  loader::RepositoryStateLoader,
  models::{IndexMetadata, IndexMetadataView, Slot},
  msg::SelectResponse,
  state::{
    ACL_CONTRACT_ADDR, ALLOWED_CODE_IDS, COUNT, CREATED_BY, DEFAULT_CODE_ID, DEFAULT_LABEL,
    IX_META_BOOL, IX_META_STRING, IX_META_TIMESTAMP, IX_META_U128, IX_META_U64, PRESETS,
  },
};

pub fn select(
  deps: Deps,
  fields: Option<Vec<String>>,
  wallet: Option<Addr>,
) -> Result<SelectResponse, ContractError> {
  let loader = RepositoryStateLoader::new(deps.storage, &fields);
  Ok(SelectResponse {
    count: loader.get("count", &COUNT)?,
    created_by: loader.get("created_by", &CREATED_BY)?,
    default_label: loader.get("default_label", &DEFAULT_LABEL)?,
    default_code_id: loader.get("default_code_id", &DEFAULT_CODE_ID)?,
    acl_address: loader.get("acl_address", &ACL_CONTRACT_ADDR)?,
    presets: loader.view_by_wallet("presets", wallet, |wallet| {
      Ok(Some(
        PRESETS
          .prefix(wallet.clone())
          .range(deps.storage, None, None, Order::Ascending)
          .map(|entry| {
            let (_k, v) = entry.unwrap();
            v
          })
          .collect(),
      ))
    })?,
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
        uint64: collect_values(deps.storage, &IX_META_U64)?,
        uint128: collect_values(deps.storage, &IX_META_U128)?,
        text: collect_values(deps.storage, &IX_META_STRING)?,
        boolean: collect_values(deps.storage, &IX_META_BOOL)?,
        timestamp: collect_values(deps.storage, &IX_META_TIMESTAMP)?,
      }))
    })?,
  })
}

fn collect_values<'a>(
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
