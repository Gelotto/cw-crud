use crate::error::ContractError;
use crate::models::ContractMetadata;
use crate::msg::QueryMsg;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{
  self, ADDR_2_ID, ID_2_ADDR, IX_CODE_ID, IX_CREATED_AT, IX_HEIGHT, IX_REV, IX_UPDATED_AT, METADATA,
};
use crate::{execute, query};
use cosmwasm_std::{
  entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response,
};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = "cw-repo";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
  deps: DepsMut,
  env: Env,
  info: MessageInfo,
  msg: InstantiateMsg,
) -> Result<Response, ContractError> {
  set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
  state::initialize(deps, &env, &info, &msg)?;
  Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
  deps: DepsMut,
  env: Env,
  info: MessageInfo,
  msg: ExecuteMsg,
) -> Result<Response, ContractError> {
  match msg {
    ExecuteMsg::Create {
      code_id,
      msg: instantiate_msg,
      admin,
      label,
      indices,
      preset,
      tags,
    } => execute::create(
      deps,
      env,
      info,
      code_id,
      &instantiate_msg,
      admin,
      label,
      indices,
      preset,
      tags,
    ),
    ExecuteMsg::CreateFromPreset {
      owner: preset_owner,
      preset,
      code_id,
      msg: instantiate_msg,
      admin,
      label,
      indices,
      tags,
    } => execute::create_from_preset(
      deps,
      env,
      info,
      code_id,
      instantiate_msg,
      admin,
      label,
      indices,
      preset_owner,
      preset,
      tags,
    ),
    ExecuteMsg::Update {
      values,
      relationships,
      tags,
    } => execute::update(deps, env, info, values, relationships, tags),
    ExecuteMsg::RenameIndex { name } => execute::rename_index(deps, env, info, name),
    ExecuteMsg::Remove { contract_addr } => execute::remove(deps, env, info, &contract_addr),
    ExecuteMsg::SetAcl { acl_contract_addr } => {
      execute::set_acl(deps, env, info, &acl_contract_addr)
    },
    ExecuteMsg::UpdateAllowedCodeIdes { code_ids } => {
      execute::update_allowed_code_ids(deps, env, info, code_ids)
    },
    ExecuteMsg::RemovePreset {
      preset: preset_name,
    } => execute::remove_preset(deps, env, info, &preset_name),
  }
}

#[entry_point]
pub fn query(
  deps: Deps,
  _env: Env,
  msg: QueryMsg,
) -> Result<Binary, ContractError> {
  let result = match msg {
    QueryMsg::Select { wallet, fields } => to_binary(&query::select(deps, fields, wallet)?),
    QueryMsg::Values { contract_addr } => to_binary(&query::values(deps, &contract_addr)?),
    QueryMsg::Count {} => to_binary(&query::count(deps)?),
    QueryMsg::Read {
      target,
      cursor,
      limit,
      desc,
      fields,
      since,
      meta,
      wallet,
    } => to_binary(&query::read(
      deps, &target, desc, limit, fields, since, meta, wallet, cursor,
    )?),
  }?;
  Ok(result)
}

#[entry_point]
pub fn reply(
  deps: DepsMut,
  env: Env,
  reply: Reply,
) -> Result<Response, ContractError> {
  match &reply.result {
    cosmwasm_std::SubMsgResult::Ok(subcall_resp) => {
      let contract_id = reply.id;
      if let Some(e) = subcall_resp.events.iter().find(|e| e.ty == "instantiate") {
        if let Some(attr) = e
          .attributes
          .iter()
          .find(|attr| attr.key == "_contract_address")
        {
          let contract_addr = Addr::unchecked(attr.value.to_string());
          let rev: u64 = 0;

          if METADATA.has(deps.storage, contract_addr.clone()) {
            return Ok(Response::default());
          }

          let mut contract_code_id = 0u64;

          if let Some(attr) = e.attributes.iter().find(|attr| attr.key == "code_id") {
            if let Ok(code_id) = u64::from_str_radix(&attr.value, 10) {
              IX_CODE_ID.save(deps.storage, (code_id, contract_id), &true)?;
              contract_code_id = code_id;
            } else {
              return Err(ContractError::CreateFailed {});
            }
          }

          IX_REV.save(deps.storage, (rev, contract_id), &true)?;
          IX_CREATED_AT.save(deps.storage, (env.block.time.nanos(), contract_id), &true)?;
          IX_UPDATED_AT.save(deps.storage, (env.block.time.nanos(), contract_id), &true)?;
          IX_HEIGHT.save(deps.storage, (env.block.height, contract_id), &true)?;

          ID_2_ADDR.save(deps.storage, reply.id, &contract_addr)?;
          ADDR_2_ID.save(deps.storage, contract_addr.clone(), &reply.id)?;

          METADATA.save(
            deps.storage,
            contract_addr.clone(),
            &ContractMetadata {
              id: reply.id,
              code_id: contract_code_id,
              height: env.block.height,
              created_at: env.block.time,
              updated_at: env.block.time,
              rev,
            },
          )?;

          deps.api.debug(
            format!(
              "created contract: {} at time {} with id {}",
              contract_addr, env.block.time, reply.id
            )
            .as_str(),
          );
        }
      }
    },
    cosmwasm_std::SubMsgResult::Err(err_reason) => {
      deps
        .api
        .debug(format!("execute of 'create' submsg error: {}", err_reason).as_str());
      return Err(ContractError::CreateFailed {});
    },
  }
  Ok(Response::default())
}
