network ?= devnet  # network := devnet|mainnet|testnet
contract_addr_filepath ?= $(release_dirpath)/contract_addr.txt
wasm_filename ?= cw_repo.wasm
release_dirpath ?= ./release
sender ?= juno16g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y
desc ?= false

# build optimized WASM artifact
build:
	./bin/build

# deploy WASM file (generated from `make build`)
deploy:
	./bin/deploy ./artifacts/$(wasm_filename) $(network) $(sender)

# instantiate last contract to be deployed using code ID in release dir code-id file
instantiate:
	./bin/instantiate $(network) $(sender) $(acl_contract_addr) $(allowed_code_id)

# run all unit tests
test:
	RUST_BACKTRACE=1 cargo unit-test

# Generate the contract's JSONSchema JSON files in schemas/
schemas:
	cargo schema

# Run/start local "devnet" validator docker image	
validator:
	./bin/validator

execute-create:
	./client.sh create $(network) $(contract_addr_filepath) $(sender) $(code_id) $(instantiate_msg)

execute-enable-acl:
	./client.sh enable-acl $(network) $(contract_addr_filepath) $(sender)

query-count:
	./client.sh count $(network) $(contract_addr_filepath) $(sender)

query-select:
	./client.sh select $(network) $(contract_addr_filepath)

query-read:
	./client.sh read $(network) $(contract_addr_filepath) $(index) $(desc)

query-read-custom-index:
	./client.sh read-custom-index $(network) $(contract_addr_filepath) $(index_type) $(index_name) $(desc)