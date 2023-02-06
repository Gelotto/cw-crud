use cosmwasm_std::{to_binary, Addr, Binary, Empty, QuerierWrapper, StdResult, Storage, WasmMsg};
use cw_storage_plus::Item;
use serde::{de::DeserializeOwned, Serialize};

use crate::{
  loader::RepositoryStateLoader,
  models::{ContractID, IndexBounds, IndexSlotValue},
  msg::{ExecuteMsg, QueryMsg, Since},
};

pub struct Repository {
  pub contract_addr: Addr,
}

impl Repository {
  pub fn new(repo_contract_addr: &Addr) -> Self {
    Self {
      contract_addr: repo_contract_addr.clone(),
    }
  }

  pub fn loader<'a>(
    storage: &'a dyn Storage,
    fields: &Option<Vec<String>>,
  ) -> RepositoryStateLoader<'a> {
    RepositoryStateLoader::new(storage, fields)
  }

  pub fn select(
    &self,
    querier: &QuerierWrapper<Empty>,
    index: &IndexBounds,
    desc: Option<bool>,
    limit: Option<u32>,
    include: Option<Vec<String>>,
    since: Option<Since>,
    meta: Option<bool>,
    cursor: Option<(String, ContractID)>,
  ) -> StdResult<Binary> {
    querier.query_wasm_smart(
      self.contract_addr.clone(),
      &QueryMsg::Read {
        index: index.clone(),
        desc,
        limit,
        fields: include,
        since,
        meta,
        cursor,
      },
    )
  }

  pub fn update(
    &self,
    values: Vec<IndexSlotValue>,
  ) -> StdResult<WasmMsg> {
    Ok(WasmMsg::Execute {
      contract_addr: self.contract_addr.clone().into(),
      funds: vec![],
      msg: to_binary(&ExecuteMsg::Update {
        values: Some(
          values
            .iter()
            .map(|v| {
              if let IndexSlotValue::Text { slot, value } = v {
                IndexSlotValue::Text {
                  slot: *slot,
                  value: Binary::from(value.as_bytes()).to_base64(),
                }
              } else {
                v.clone()
              }
            })
            .collect(),
        ),
      })?,
    })
  }

  pub fn may_load_item<'a, T>(
    storage: &dyn Storage,
    fields: &Option<Vec<String>>,
    field_name: &str,
    item: &Item<'a, T>,
  ) -> StdResult<Option<T>>
  where
    T: DeserializeOwned,
    T: Serialize,
  {
    if let Some(fields) = fields {
      if fields.is_empty() || fields.contains(&field_name.to_owned()) {
        item.may_load(storage)
      } else {
        Ok(None)
      }
    } else {
      Ok(None)
    }
  }
}
