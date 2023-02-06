#!/bin/bash

CMD=$1
NETWORK=$2
CONTRACT_ADDR=$(cat $3)
NODE=
CHAIN_ID=
FLAGS=

shift 3

case $NETWORK in
  testnet)
    NODE="https://rpc.uni.juno.deuslabs.fi:443"
    CHAIN_ID=uni-3
    DENOM=ujunox
    ;;
  mainnet)
    NODE="https://rpc-juno.itastakers.com",
    CHAIN_ID=juno-1
    DENOM=ujuno
    ;;
  devnet)
    NODE="http://localhost:26657"
    CHAIN_ID=testing
    DENOM=ujunox
    ;;
esac


create() {
  sender=$1
  instantiate_msg=$2
  b64_instantiate_msg="$(echo $instantiate_msg | ./bin/utils/b64-encode)"
  b64_lucky_phrase="$(echo hello | ./bin/utils/b64-encode)"
  # msg='{"create":{"msg":"'$b64_instantiate_msg'","indices":[{"number":{"slot":0,"value":0}},{"text":{"slot":1,"value":"'$b64_lucky_phrase'"}}]}}'
  msg='{"create":{"msg":"'$b64_instantiate_msg'"}}'
  flags="\
  --node $NODE \
  --gas-prices 0.025$DENOM \
  --chain-id $CHAIN_ID \
  --from $sender \
  --gas auto \
  --gas-adjustment 1.5 \
  --broadcast-mode block \
  --output json \
  -y \
  "
  echo junod tx wasm execute $CONTRACT_ADDR "'$msg'" "$flags"
  response=$(junod tx wasm execute "$CONTRACT_ADDR" "$msg" $flags)
  echo $response | ./bin/utils/base64-decode-attributes | jq
}

enable-acl() {
  sender=$1
  msg='{"enable_acl":{}}'
  flags="\
  --node $NODE \
  --gas-prices 0.025$DENOM \
  --chain-id $CHAIN_ID \
  --from $sender \
  --gas auto \
  --gas-adjustment 1.5 \
  --broadcast-mode block \
  --output json \
  -y \
  "
  echo junod tx wasm execute $CONTRACT_ADDR "$msg" "$flags"
  response=$(junod tx wasm execute "$CONTRACT_ADDR" "$msg" $flags)
  echo $response | ./bin/utils/base64-decode-attributes | jq
}


count() {
  query='{"count":{}}'
  flags="--chain-id $CHAIN_ID --output json --node $NODE"
  echo junod query wasm contract-state smart $CONTRACT_ADDR "$query" $flags
  response=$(junod query wasm contract-state smart $CONTRACT_ADDR "$query" $flags)
  echo $response | ./bin/utils/base64-decode-attributes | jq
}

select_query() {
  query='{"select":{"fields":[]}}'
  flags="--chain-id $CHAIN_ID --output json --node $NODE"
  echo junod query wasm contract-state smart $CONTRACT_ADDR "$query" $flags
  response=$(junod query wasm contract-state smart $CONTRACT_ADDR "$query" $flags)
  echo $response | ./bin/utils/base64-decode-attributes | jq
}

read_index() {
  index=$1
  desc=$2
  limit=$3
  query='{"read":{"index":{"'$index'":{}},"desc":'$desc',"fields":[],"limit":'$limit',"meta":false}}'
  flags="--chain-id $CHAIN_ID --output json --node $NODE"
  echo junod query wasm contract-state smart $CONTRACT_ADDR "$query" $flags
  response=$(junod query wasm contract-state smart $CONTRACT_ADDR "$query" $flags)
  echo $response | ./bin/utils/b64-decode-state | jq
}

read_string_index() {
  slot=$1
  desc=$2
  equals=$3
  if [ -z "$equals" ]; then
    query='{"read":{"index":{"text":{"slot":'$slot'}},"fields":[],"desc":'$desc',"meta":true}}'
  else
    b64_equals="$(echo "$equals" | ./bin/utils/b64-encode)"
    query='{"read":{"index":{"text":{"slot":'$slot',"equals":"'$b64_equals'"}},"fields":[],"desc":'$desc',"meta":false}}'
  fi
  flags="--chain-id $CHAIN_ID --output json --node $NODE"
  echo junod query wasm contract-state smart $CONTRACT_ADDR "$query" $flags
  response=$(junod query wasm contract-state smart $CONTRACT_ADDR "$query" $flags)
  echo $response | ./bin/utils/b64-decode-state | jq
}


set -e

case $CMD in
  create)
    create $1 $2
    ;;
  enable-acl)
    enable-acl $1
    ;;
  count) 
    count
    ;;
  select) 
    select_query
    ;;
  read) 
    read_index $1 $2 $3
    ;;
  read-string-index)
    read_string_index $1 $2 $3 $4
    ;;
esac