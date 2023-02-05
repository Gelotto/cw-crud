use std::{collections::HashSet, iter::FromIterator};

use cosmwasm_std::{to_binary, Addr, Binary, Empty, QuerierWrapper, StdResult, Storage, WasmMsg};
use cw_storage_plus::Item;
use serde::{de::DeserializeOwned, Serialize};

use crate::{
  models::{ContractID, IndexBounds, IndexUpdate},
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
    indices: Vec<IndexUpdate>,
  ) -> StdResult<WasmMsg> {
    Ok(WasmMsg::Execute {
      contract_addr: self.contract_addr.clone().into(),
      msg: to_binary(&ExecuteMsg::UpdateIndices {
        values: Some(indices.clone()),
      })?,
      funds: vec![],
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

pub struct RepositoryStateLoader<'a> {
  storage: &'a dyn Storage,
  fields: HashSet<String>,
}

impl<'a> RepositoryStateLoader<'a> {
  pub fn new(
    storage: &'a dyn Storage,
    fields: &Option<Vec<String>>,
  ) -> Self {
    Self {
      storage: storage,
      fields: if let Some(fields_vec) = fields {
        HashSet::from_iter(fields_vec.iter().map(|x| x.clone()))
      } else {
        HashSet::new()
      },
    }
  }

  pub fn get<'b, T>(
    &self,
    field: &str,
    item: &Item<'b, T>,
  ) -> StdResult<Option<T>>
  where
    T: DeserializeOwned,
    T: Serialize,
  {
    if self.fields.is_empty() || self.fields.contains(&field.to_owned()) {
      item.may_load(self.storage)
    } else {
      Ok(None)
    }
  }

  pub fn view<T, F>(
    &self,
    field: &str,
    func: F,
  ) -> StdResult<Option<T>>
  where
    F: Fn() -> StdResult<Option<T>>,
  {
    if self.fields.is_empty() || self.fields.contains(&field.to_owned()) {
      func()
    } else {
      Ok(None)
    }
  }
}
