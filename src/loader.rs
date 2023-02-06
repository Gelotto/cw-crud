use std::{collections::HashSet, iter::FromIterator};

use cosmwasm_std::{StdResult, Storage};
use cw_storage_plus::Item;
use serde::{de::DeserializeOwned, Serialize};

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
