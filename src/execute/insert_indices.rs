use crate::{
  error::ContractError,
  models::IndexSlotValue,
  state::{
    get_bool_index, get_contract_id, get_number_index, get_text_index, get_timestamp_index,
    update_index_metadata_count, BOOL_INDEX_METADATA, NUMBER_INDEX_METADATA, TS_INDEX_METADATA,
  },
  state::{owns_contract, TEXT_INDEX_METADATA},
};
use cosmwasm_std::{attr, Attribute, DepsMut, Env, MessageInfo, Response};

pub fn insert_indices(
  deps: DepsMut,
  _env: Env,
  info: MessageInfo,
  values: Vec<IndexSlotValue>,
) -> Result<Response, ContractError> {
  let contract_addr = &info.sender;

  if !owns_contract(deps.storage, contract_addr) {
    return Err(ContractError::NotAuthorized {});
  }

  let contract_id = get_contract_id(deps.storage, contract_addr)?;
  let mut attrs: Vec<Attribute> = vec![
    attr("action", "index"),
    attr("contract_id", contract_id.to_string()),
  ];

  for value in values.iter() {
    match value.clone() {
      IndexSlotValue::Number { slot, value } => {
        deps
          .api
          .debug(format!("setting value in u64 index, slot {}", slot).as_str());

        let map = get_number_index(slot)?;
        let key = (value, contract_id);

        map.save(deps.storage, key, &true)?;
        update_index_metadata_count(deps.storage, &NUMBER_INDEX_METADATA, slot)?;
        attrs.push(attr("slot", slot.to_string()));
        attrs.push(attr("value", value.to_string()));
      },
      IndexSlotValue::Timestamp { slot, value } => {
        deps
          .api
          .debug(format!("setting value in timestamp index, slot {}", slot).as_str());

        let map = get_timestamp_index(slot)?;
        let key = (value.nanos(), contract_id);

        map.save(deps.storage, key, &true)?;
        update_index_metadata_count(deps.storage, &TS_INDEX_METADATA, slot)?;
        attrs.push(attr("slot", slot.to_string()));
        attrs.push(attr("value", value.to_string()));
      },
      IndexSlotValue::Text { slot, value } => {
        deps
          .api
          .debug(format!("setting value in text index, slot {}", slot).as_str());

        let map = get_text_index(slot)?;
        let key = (value.clone(), contract_id);

        map.save(deps.storage, key, &true)?;
        update_index_metadata_count(deps.storage, &TEXT_INDEX_METADATA, slot)?;
        attrs.push(attr("slot", slot.to_string()));
        attrs.push(attr("value", value.to_string()));
      },
      IndexSlotValue::Boolean { slot, value } => {
        deps
          .api
          .debug(format!("setting value in bool index, slot {}", slot).as_str());

        let map = get_bool_index(slot)?;
        let key = (if value { 1 } else { 0 }, contract_id);

        map.save(deps.storage, key, &true)?;
        update_index_metadata_count(deps.storage, &BOOL_INDEX_METADATA, slot)?;
        attrs.push(attr("slot", slot.to_string()));
        attrs.push(attr("value", value.to_string()));
      },
    }
  }

  Ok(Response::new().add_attributes(attrs))
}
