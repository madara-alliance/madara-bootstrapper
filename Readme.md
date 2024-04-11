# Karnot Bridge Deploy

Karnot bridge deploy is a tool that will deploy the bridge contract on settlement layer and on the rollup side.

```sh
ARGS:
//////////////////////////////////////////////////

- eth_rpc
- eth_priv_key
- rollup_sequencer_url
- rollup_priv_key
- L1 deployer address : will be used as a governor in bridge contracts
- L2 deployer address
```

## To test

```sh
# 1. Run madara instance with eth as settlement layer :
./target/release/madara --dev --sealing=manual --da-layer=ethereum --da-conf=examples/da-confs/ethereum.json --settlement=Ethereum --settlement-conf=examples/da-confs/ethereum.json

# 2. Run anvil instance
~/.foundry/bin/anvil

# 3. Run tests
# To test deployment of bridges :
cargo test deploy_bridge -- --nocapture --exact

# To test eth deposit and withdrawal
cargo test deposit_and_withdraw_eth_bridge -- --nocapture --exact

# To test erc20 deposit and withdrawal
cargo test deposit_and_withdraw_erc20_bridge -- --nocapture --exact
```
