use cosmwasm_std::{to_binary, Addr, Binary, Empty, QuerierWrapper, StdResult, WasmMsg};

use crate::{
  models::{IndexSelection, IndexUpdate},
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

  pub fn select(
    &self,
    querier: &QuerierWrapper<Empty>,
    ix: &IndexSelection,
    desc: Option<bool>,
    limit: Option<u32>,
    include: Option<Vec<String>>,
    since: Option<Since>,
    meta: Option<bool>,
  ) -> StdResult<Binary> {
    querier.query_wasm_smart(
      self.contract_addr.clone(),
      &QueryMsg::ExecuteSelect {
        index: ix.clone(),
        desc,
        limit,
        include,
        since,
        meta,
      },
    )
  }

  pub fn update(
    &self,
    indices: Vec<IndexUpdate>,
  ) -> StdResult<WasmMsg> {
    Ok(WasmMsg::Execute {
      contract_addr: self.contract_addr.clone().into(),
      msg: to_binary(&ExecuteMsg::Update {
        indices: Some(indices.clone()),
      })?,
      funds: vec![],
    })
  }
}
