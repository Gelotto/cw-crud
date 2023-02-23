use cosmwasm_std::{
  to_binary, Addr, Binary, Empty, QuerierWrapper, StdResult, Storage, Timestamp, WasmMsg,
};

use crate::{
  loader::RepositoryStateLoader,
  models::{ContractID, IndexSlotValue, Relationship, RelationshipUpdates, Slot, TagUpdates},
  msg::{ExecuteMsg, QueryMsg, Since, Target},
};

#[derive(Clone)]
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

  pub fn update(&self) -> UpdateBuilder {
    UpdateBuilder::new(&self.contract_addr)
  }

  pub fn select(
    &self,
    querier: &QuerierWrapper<Empty>,
    target: &Target,
    desc: Option<bool>,
    limit: Option<u32>,
    include: Option<Vec<String>>,
    since: Option<Since>,
    meta: Option<bool>,
    wallet: Option<Addr>,
    cursor: Option<(String, ContractID)>,
  ) -> StdResult<Binary> {
    querier.query_wasm_smart(
      self.contract_addr.clone(),
      &QueryMsg::Read {
        target: target.clone(),
        desc,
        limit,
        fields: include,
        since,
        meta,
        cursor,
        wallet,
      },
    )
  }
}

#[derive(Clone)]
pub struct UpdateBuilder {
  repo_contract_addr: Addr,
  values: Vec<IndexSlotValue>,
  addr_tags_to_add: Vec<Relationship>,
  addr_tags_to_delete: Vec<Relationship>,
  tags_to_add: Vec<String>,
  tags_to_delete: Vec<String>,
}

impl UpdateBuilder {
  pub fn new(repo_contract_addr: &Addr) -> Self {
    Self {
      repo_contract_addr: repo_contract_addr.clone(),
      values: vec![],
      tags_to_add: vec![],
      tags_to_delete: vec![],
      addr_tags_to_add: vec![],
      addr_tags_to_delete: vec![],
    }
  }

  pub fn set_u64(
    mut self,
    slot: Slot,
    value: u64,
  ) -> Self {
    self.values.push(IndexSlotValue::Uint64 { slot, value });
    self
  }

  pub fn set_u128(
    mut self,
    slot: Slot,
    value: u128,
  ) -> Self {
    self.values.push(IndexSlotValue::Uint128 { slot, value });
    self
  }

  pub fn set_string(
    mut self,
    slot: Slot,
    value: &String,
  ) -> Self {
    self.values.push(IndexSlotValue::Text {
      slot,
      value: value.clone(),
    });
    self
  }

  pub fn set_boolean(
    mut self,
    slot: Slot,
    value: bool,
  ) -> Self {
    self.values.push(IndexSlotValue::Boolean { slot, value });
    self
  }

  pub fn set_timestamp(
    mut self,
    slot: Slot,
    value: Timestamp,
  ) -> Self {
    self.values.push(IndexSlotValue::Timestamp { slot, value });
    self
  }

  pub fn tag(
    mut self,
    tag: impl Into<String>,
  ) -> Self {
    self.tags_to_add.push(tag.into());
    self
  }

  pub fn untag(
    mut self,
    tag: impl Into<String>,
  ) -> Self {
    self.tags_to_delete.push(tag.into());
    self
  }

  pub fn tag_address(
    mut self,
    addr: &Addr,
    tag: impl Into<String>,
  ) -> Self {
    self.addr_tags_to_add.push(Relationship {
      address: addr.clone(),
      tag: tag.into(),
    });
    self
  }

  pub fn untag_address(
    mut self,
    addr: &Addr,
    tag: impl Into<String>,
  ) -> Self {
    self.addr_tags_to_delete.push(Relationship {
      address: addr.clone(),
      tag: tag.into(),
    });
    self
  }

  pub fn build_msg(&self) -> StdResult<WasmMsg> {
    let values = if !self.values.is_empty() {
      Some(self.values.clone())
    } else {
      None
    };
    let rel_updates = Some(RelationshipUpdates {
      added: if !self.addr_tags_to_add.is_empty() {
        Some(self.addr_tags_to_add.clone())
      } else {
        None
      },
      removed: if !self.addr_tags_to_delete.is_empty() {
        Some(self.addr_tags_to_delete.clone())
      } else {
        None
      },
    });
    let tags = Some(TagUpdates {
      added: if !self.tags_to_add.is_empty() {
        Some(self.tags_to_add.clone())
      } else {
        None
      },
      removed: if !self.tags_to_delete.is_empty() {
        Some(self.tags_to_delete.clone())
      } else {
        None
      },
    });
    Ok(WasmMsg::Execute {
      contract_addr: self.repo_contract_addr.clone().into(),
      funds: vec![],
      msg: to_binary(&ExecuteMsg::Update {
        relationships: rel_updates,
        tags,
        // base64 encode string values
        values: Some(
          values
            .unwrap_or(vec![])
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
}
