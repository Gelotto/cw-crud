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
  code_id=$2
  msg='{"create":{"code_id":'$code_id',"instantiate_msg":"e30="}}'
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

read_index() {
  index=$1
  desc=$2
  query='{"read":{"index":{"'$index'":{}},"desc":'$desc',"verbose":true}}'
  flags="--chain-id $CHAIN_ID --output json --node $NODE"
  echo junod query wasm contract-state smart $CONTRACT_ADDR "$query" $flags
  response=$(junod query wasm contract-state smart $CONTRACT_ADDR "$query" $flags)
  echo $response | ./bin/utils/base64-decode-attributes | jq
}

set -e

echo $*
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
  read) 
    read_index $1 $2
    ;;
esac