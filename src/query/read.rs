use std::marker::PhantomData;

use cosmwasm_std::{Addr, Binary, Deps, Order, StdError, StdResult, Storage, Timestamp};
use cw_storage_plus::{Bound, Map, PrefixBound};

use crate::{
  error::ContractError,
  models::{ContractID, IndexSelection},
  msg::{ContractStateEnvelope, GetState, ReadResponse},
  state::{
    get_str_index, get_u64_index, ID_2_ADDR, IX_CODE_ID, IX_CREATED_AT, IX_REV, IX_UPDATED_AT,
    METADATA,
  },
};

pub const MIN_LIMIT: u32 = 1;
pub const MAX_LIMIT: u32 = 50;
pub const DEFAULT_LIMIT: u32 = 25;

/// Return total number of contracts in the repo.
pub fn read(
  deps: Deps,
  target: &IndexSelection,
  some_limit: Option<u32>,
  desc: Option<bool>,
  params: Option<Binary>,
  some_modified_since: Option<Timestamp>,
  verbose: Option<bool>,
) -> Result<ReadResponse, ContractError> {
  // clamp limit to min and max bounds
  let limit = some_limit
    .unwrap_or(DEFAULT_LIMIT)
    .clamp(MIN_LIMIT, MAX_LIMIT);

  // resolve Order enum from desc flag
  let order = if desc.unwrap_or(false) {
    Order::Descending
  } else {
    Order::Ascending
  };

  // we normalize the bound types for each target index to strings so we can
  // return them in a simplified manner in the pagination response envelope
  let mut bounds: Vec<Option<String>> = vec![None, None];

  // compute vec of contract ID's from an index
  let contract_ids: Vec<ContractID> = match target.clone() {
    IndexSelection::CreatedAt { start, stop } => {
      bounds[0] = start.clone().and_then(|x| Some(x.nanos().to_string()));
      bounds[1] = stop.clone().and_then(|x| Some(x.nanos().to_string()));
      paginate_timestamp_index(deps.storage, &IX_CREATED_AT, start, stop, order, limit)?
    },
    IndexSelection::UpdatedAt { start, stop } => {
      bounds[0] = start.clone().and_then(|x| Some(x.nanos().to_string()));
      bounds[1] = stop.clone().and_then(|x| Some(x.nanos().to_string()));
      paginate_timestamp_index(deps.storage, &IX_UPDATED_AT, start, stop, order, limit)?
    },
    IndexSelection::Address { start, stop } => {
      bounds[0] = start.clone().and_then(|x| Some(x.to_string()));
      bounds[1] = stop.clone().and_then(|x| Some(x.to_string()));
      paginate_metadata(deps.storage, start, stop, order, limit)?
    },
    IndexSelection::Revision { start, stop } => {
      bounds[0] = start.clone().and_then(|x| Some(x.to_string()));
      bounds[1] = stop.clone().and_then(|x| Some(x.to_string()));
      paginate_u64_index_map(deps.storage, &IX_REV, start, stop, order, limit)?
    },
    IndexSelection::CodeId { start, stop } => {
      bounds[0] = start.clone().and_then(|x| Some(x.to_string()));
      bounds[1] = stop.clone().and_then(|x| Some(x.to_string()));
      paginate_u64_index_map(deps.storage, &IX_CODE_ID, start, stop, order, limit)?
    },
    IndexSelection::Numeric { idx, start, stop } => {
      bounds[0] = start.clone().and_then(|x| Some(x.to_string()));
      bounds[1] = stop.clone().and_then(|x| Some(x.to_string()));
      paginate_u64_index(deps.storage, idx, start, stop, order, limit)?
    },
    IndexSelection::Text { idx, start, stop } => {
      bounds[0] = start.clone();
      bounds[1] = stop.clone();
      paginate_str_index(deps.storage, idx, start, stop, order, limit)?
    },
  };

  // build vec of returned contract addresses from contract ID's, along with
  // any queried state from each contract, provided params is not None.
  let mut page: Vec<ContractStateEnvelope> = Vec::with_capacity(contract_ids.len());
  for id in contract_ids.iter() {
    let contract_addr = ID_2_ADDR.load(deps.storage, *id)?;
    let some_meta = if verbose.unwrap_or(false) {
      METADATA.may_load(deps.storage, contract_addr.clone())?
    } else {
      None
    };

    //skip if not modified since modified_since timestamp
    if let Some(modified_since) = some_modified_since {
      let meta = if let Some(meta) = &some_meta {
        meta.clone()
      } else {
        METADATA.load(deps.storage, contract_addr.clone())?
      };
      if meta.updated_at <= modified_since {
        continue;
      }
    }

    // query state from contract via its GetState API implementation
    let state = match &params {
      None => None,
      Some(params) => deps.querier.query_wasm_smart(
        contract_addr.clone(),
        &GetState {
          params: params.clone(),
        },
      )?,
    };

    page.push(ContractStateEnvelope {
      address: contract_addr.clone(),
      meta: some_meta,
      state,
    })
  }

  Ok(ReadResponse {
    count: page.len() as u8,
    start: bounds[0].clone(),
    stop: bounds[1].clone(),
    page,
  })
}

fn paginate_metadata(
  store: &dyn Storage,
  start: Option<Addr>,
  stop: Option<Addr>,
  order: Order,
  limit: u32,
) -> Result<Vec<ContractID>, ContractError> {
  let map = METADATA;

  let start_bound = start
    .and_then(|x| Some(Bound::Inclusive((x, PhantomData))))
    .or(None);

  let stop_bound = stop
    .and_then(|x| Some(Bound::Exclusive((x, PhantomData))))
    .or(None);

  let iter = map.range(store, start_bound, stop_bound, order);

  return collect(iter, limit, |_, v| -> Result<ContractID, ContractError> {
    Ok(v.id)
  });
}

fn paginate_timestamp_index<'a>(
  store: &dyn Storage,
  map: &Map<'a, (u64, ContractID), bool>,
  start: Option<Timestamp>,
  stop: Option<Timestamp>,
  order: Order,
  limit: u32,
) -> Result<Vec<ContractID>, ContractError> {
  paginate_u64_index_map(
    store,
    map,
    start.and_then(|x| Some(x.nanos())).or(None),
    stop.and_then(|x| Some(x.nanos())).or(None),
    order,
    limit,
  )
}

fn paginate_u64_index_map<'a>(
  store: &dyn Storage,
  map: &Map<'a, (u64, ContractID), bool>,
  start: Option<u64>,
  stop: Option<u64>,
  order: Order,
  limit: u32,
) -> Result<Vec<ContractID>, ContractError> {
  let start_bound = start
    .and_then(|n| Some(PrefixBound::Inclusive((n, PhantomData))))
    .or(None);

  let stop_bound = stop
    .and_then(|n| Some(PrefixBound::Exclusive((n, PhantomData))))
    .or(None);

  let iter = map.prefix_range(store, start_bound, stop_bound, order);

  return collect(iter, limit, |k, _| -> Result<ContractID, ContractError> {
    Ok(k.1)
  });
}

fn paginate_u64_index(
  store: &dyn Storage,
  idx: u8,
  start: Option<u64>,
  stop: Option<u64>,
  order: Order,
  limit: u32,
) -> Result<Vec<ContractID>, ContractError> {
  let map = get_u64_index(idx)?;

  let start_bound = start
    .and_then(|n| Some(PrefixBound::Inclusive((n, PhantomData))))
    .or(None);

  let stop_bound = stop
    .and_then(|n| Some(PrefixBound::Exclusive((n, PhantomData))))
    .or(None);

  let iter = map.prefix_range(store, start_bound, stop_bound, order);

  return collect(iter, limit, |k, _| -> Result<ContractID, ContractError> {
    Ok(k.1)
  });
}

fn paginate_str_index(
  store: &dyn Storage,
  idx: u8,
  start: Option<String>,
  stop: Option<String>,
  order: Order,
  limit: u32,
) -> Result<Vec<ContractID>, ContractError> {
  let map = get_str_index(idx)?;

  let start_bound = start
    .and_then(|n| Some(PrefixBound::Inclusive((n, PhantomData))))
    .or(None);

  let stop_bound = stop
    .and_then(|n| Some(PrefixBound::Exclusive((n, PhantomData))))
    .or(None);

  let iter = map.prefix_range(store, start_bound, stop_bound, order);

  return collect(iter, limit, |k, _| -> Result<ContractID, ContractError> {
    Ok(k.1)
  });
}

pub fn collect<'a, D, T, R, E, F>(
  iter: Box<dyn Iterator<Item = StdResult<(D, T)>> + 'a>,
  limit: u32,
  parse_fn: F,
) -> Result<Vec<R>, E>
where
  F: Fn(D, T) -> Result<R, E>,
  E: From<StdError>,
{
  let limit = limit as usize;
  iter
    .take(limit)
    .map(|item| {
      let (k, v) = item?;
      parse_fn(k, v)
    })
    .collect()
}

// TODO: move into cw-lib utils
// pub fn paginate_map<'a, K, T, R, E, F>(
//   map: &Map<'a, K, T>,
//   store: &dyn Storage,
//   start: Option<Bound<'a, K>>,
//   stop: Option<Bound<'a, K>>,
//   order: Order,
//   limit: u32,
//   parse_fn: F,
// ) -> Result<Vec<R>, E>
// where
//   K: PrimaryKey<'a> + KeyDeserialize,
//   K::Output: 'static,
//   T: Serialize + DeserializeOwned,
//   F: Fn(K::Output, T) -> Result<R, E>,
//   E: From<StdError>,
// {
//   let iter = map.range(store, start, stop, order);
//   return collect(iter, limit, parse_fn);
// }
//
// /// Iterate entries in a `cw_storage_plus::Map` under a given prefix.
// pub fn paginate_map_prefix<'a, K, T, R, E, F>(
//   map: &Map<'a, K, T>,
//   store: &dyn Storage,
//   prefix: K::Prefix,
//   start: Option<Bound<'a, K::Suffix>>,
//   stop: Option<Bound<'a, K>>,
//   order: Order,
//   limit: u32,
//   parse_fn: F,
// ) -> Result<Vec<R>, E>
// where
//   K: PrimaryKey<'a>,
//   K::Suffix: PrimaryKey<'a> + KeyDeserialize,
//   <K::Suffix as KeyDeserialize>::Output: 'static,
//   T: Serialize + DeserializeOwned,
//   F: Fn(<K::Suffix as KeyDeserialize>::Output, T) -> Result<R, E>,
//   E: From<StdError>,
// {
//   let iter = map
//     .prefix(prefix)
//     .range(store, start, None, Order::Ascending);
//   collect(iter, limit, parse_fn)
// }
//
// pub fn collect<'a, D, T, R, E, F>(
//   iter: Box<dyn Iterator<Item = StdResult<(D, T)>> + 'a>,
//   limit: u32,
//   parse_fn: F,
// ) -> Result<Vec<R>, E>
// where
//   F: Fn(D, T) -> Result<R, E>,
//   E: From<StdError>,
// {
//   let limit = limit as usize;
//   iter
//     .take(limit)
//     .map(|item| {
//       let (k, v) = item?;
//       parse_fn(k, v)
//     })
//     .collect()
// }
