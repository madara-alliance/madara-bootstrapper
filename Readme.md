# Karnot Bridge Deploy

Karnot bridge deploy is a tool that will deploy the bridge contract on settlement layer and on the rollup side.

```shell
ARGS / ENV:
//////////////////////////////////////////////////

eth_rpc
eth_priv_key
rollup_seq_url
rollup_priv_key
eth_chain_id
l1_deployer_address
l2_deployer_address
l1_wait_time
sn_os_program_hash
config_hash_version
app_chain_id
fee_token_address
```

## To test

```shell
# 1. Run madara instance with eth as settlement layer :
./target/release/madara --dev --da-layer=ethereum --da-conf=examples/da-confs/ethereum.json --settlement=Ethereum --settlement-conf=examples/da-confs/ethereum.json
# 2. Run anvil instance
~/.foundry/bin/anvil

# 3. Run tests
RUST_LOG=debug cargo test -- --no-capture
```

## To run and env setup

```shell
RUST_LOG=debug cargo run -- --help
```