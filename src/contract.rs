use crate::error::ContractError;
use crate::execute;
use crate::models::ContractMetadata;
use crate::msg::QueryMsg;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::query::{count, read, select};
use crate::state::{
  self, ADDR_2_ID, ID_2_ADDR, IX_CODE_ID, IX_CREATED_AT, IX_REV, IX_UPDATED_AT, METADATA,
};
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
      instantiate_msg,
      admin,
      funds,
      label,
      indices,
    } => execute::create::create(
      deps,
      env,
      info,
      code_id,
      &instantiate_msg,
      admin,
      funds,
      label,
      indices,
    ),
    ExecuteMsg::Update { indices } => execute::update::update(deps, env, info, indices),
  }
}

#[entry_point]
pub fn query(
  deps: Deps,
  _env: Env,
  msg: QueryMsg,
) -> Result<Binary, ContractError> {
  let result = match msg {
    QueryMsg::Select(s) => to_binary(&select(deps, s.fields)?),
    QueryMsg::Count {} => to_binary(&count(deps)?),
    QueryMsg::ExecuteSelect {
      index,
      limit,
      desc,
      include,
      since,
      meta,
    } => to_binary(&read(deps, &index, desc, limit, include, since, meta)?),
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

          if let Some(attr) = e.attributes.iter().find(|attr| attr.key == "code_id") {
            if let Ok(code_id) = u64::from_str_radix(&attr.value, 10) {
              IX_CODE_ID.save(deps.storage, (code_id, contract_id), &true)?;
            } else {
              return Err(ContractError::CreateFailed {});
            }
          }

          IX_REV.save(deps.storage, (rev, contract_id), &true)?;
          IX_CREATED_AT.save(deps.storage, (env.block.time.nanos(), contract_id), &true)?;
          IX_UPDATED_AT.save(deps.storage, (env.block.time.nanos(), contract_id), &true)?;

          ID_2_ADDR.save(deps.storage, reply.id, &contract_addr)?;
          ADDR_2_ID.save(deps.storage, contract_addr.clone(), &reply.id)?;

          METADATA.save(
            deps.storage,
            contract_addr.clone(),
            &ContractMetadata {
              id: reply.id,
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
