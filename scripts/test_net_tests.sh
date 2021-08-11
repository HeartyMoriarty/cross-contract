#!/bin/bash

near delete bank.heartymoriarty.testnet heartymoriarty.testnet
near create-account bank.heartymoriarty.testnet --masterAccount heartymoriarty.testnet

near delete token.heartymoriarty.testnet heartymoriarty.testnet
near create-account token.heartymoriarty.testnet --masterAccount heartymoriarty.testnet

near delete dude.heartymoriarty.testnet heartymoriarty.testnet
near create-account dude.heartymoriarty.testnet --masterAccount heartymoriarty.testnet

# build and place contracts in right place
cargo build --all --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/*.wasm ./res/ 

# deploy both contracts
near deploy --wasmFile res/simple_bank.wasm --accountId bank.heartymoriarty.testnet
near deploy --wasmFile res/token_contract.wasm --accountId token.heartymoriarty.testnet

# initialize both contracts with their owners
near call bank.heartymoriarty.testnet new --args "{\"owner\":\"bank.heartymoriarty.testnet\"}" --accountId bank.heartymoriarty.testnet
near call token.heartymoriarty.testnet new --args "{\"owner\":\"token.heartymoriarty.testnet\"}" --accountId token.heartymoriarty.testnet

# add accounts to token
near call token.heartymoriarty.testnet add_acc --args "{\"acc_id\":\"bank.heartymoriarty.testnet\"}" --accountId token.heartymoriarty.testnet
near call token.heartymoriarty.testnet wl_add_acc --args "{\"acc_id\":\"bank.heartymoriarty.testnet\"}" --accountId token.heartymoriarty.testnet
near call token.heartymoriarty.testnet add_acc --args "{\"acc_id\":\"dude.heartymoriarty.testnet\"}" --accountId token.heartymoriarty.testnet

# add accounts to use bank
near call bank.heartymoriarty.testnet wl_add_acc --args "{\"acc_id\":\"token.heartymoriarty.testnet\"}" --accountId bank.heartymoriarty.testnet
near call bank.heartymoriarty.testnet add_acc --args "{\"acc_id\":\"token.heartymoriarty.testnet\"}" --accountId bank.heartymoriarty.testnet
near call bank.heartymoriarty.testnet add_acc --args "{\"acc_id\":\"dude.heartymoriarty.testnet\"}" --accountId bank.heartymoriarty.testnet

# create value in dude's token account
near call token.heartymoriarty.testnet create_amount --args "{\"acc_id\":\"dude.heartymoriarty.testnet\",\"amount\":\"100\"}" --accountId token.heartymoriarty.testnet

# deposit 50 to bank
near call token.heartymoriarty.testnet transfer --args "{\"acc_id\":\"bank.heartymoriarty.testnet\",\"amount\":\"50\"}" --accountId dude.heartymoriarty.testnet --amount 0.000000000000000000000001 --gas 300000000000000

# check to see if bank received 50 and dude has 50
# first = 50, second = 50, third = 50
near call token.heartymoriarty.testnet balance_of --args "{\"acc_id\":\"bank.heartymoriarty.testnet\"}" --accountId token.heartymoriarty.testnet
near call token.heartymoriarty.testnet balance_of --args "{\"acc_id\":\"dude.heartymoriarty.testnet\"}" --accountId token.heartymoriarty.testnet
near call bank.heartymoriarty.testnet balance_of --args "{\"acc_id\":\"dude.heartymoriarty.testnet\"}" --accountId bank.heartymoriarty.testnet

# withdraw 50 from bank
near call bank.heartymoriarty.testnet transfer --args "{\"acc_id\":\"token.heartymoriarty.testnet\",\"amount\":\"50\"}" --accountId dude.heartymoriarty.testnet --amount 0.000000000000000000000001 --gas 300000000000000

# check to see if bank removed 50 and dude has 100, and bank has 0 for dude
# first = 0, second = 100, third = 0
near call token.heartymoriarty.testnet balance_of --args "{\"acc_id\":\"bank.heartymoriarty.testnet\"}" --accountId token.heartymoriarty.testnet
near call token.heartymoriarty.testnet balance_of --args "{\"acc_id\":\"dude.heartymoriarty.testnet\"}" --accountId token.heartymoriarty.testnet
near call bank.heartymoriarty.testnet balance_of --args "{\"acc_id\":\"dude.heartymoriarty.testnet\"}" --accountId bank.heartymoriarty.testnet
near call bank.heartymoriarty.testnet balance_of --args "{\"acc_id\":\"token.heartymoriarty.testnet\"}" --accountId bank.heartymoriarty.testnet