# KBD 🌉

Karnot bridge deploy (KBD) is a tool that helps to deploy
the bridge contract between a madara/katana appchain and
another L2 or L1 network.

**Currently Supported :**

- Madara App Chain <----> Ethereum / EVM based chains
- 👷🏼 more coming soon......

## Testing 🛠️

There are three test in the repository :

- bridge deployment e2e
- eth bridge deposit and claim
- erc20 token bridge deposit and claim

### IMP 🚨

- You need to comment/remove the #[ignore]
  tags in [src/tests/mod.rs](src/tests/mod.rs) file
- Only one test can be run at one time as all
  the tests are e2e tests.
- You also would need to restart
  both the chains after running each test.
- While running the erc20 bridge deposit and withdraw
  test you need to uncomment the lines 109-115 in file
  [src/contract_clients/token_bridge.rs](src/contract_clients/token_bridge.rs)

```shell
# 1. Run madara instance with eth as settlement layer :
./target/release/madara --dev --da-layer=ethereum --da-conf=examples/da-confs/ethereum.json --settlement=Ethereum --settlement-conf=examples/da-confs/ethereum.json
# 2. Run anvil instance
~/.foundry/bin/anvil

# 3. Run tests
RUST_LOG=debug cargo test -- --nocapture
```

## Run 🚀

### Local 💻

You can provide the env variables as arguments also
,or you can also provide them in .env file.

Refer [.env.example](.env.example) file for setup

```shell
cp .env.example .env
cargo build --release
RUST_LOG=debug cargo run -- --help
```

**IMP 🚨** : It will store all the addresses in [data/addresses.json](data/addresses.json)

### Docker 🐳

1. You need to set up the .env file first. Fill all
   the variables in .env file

   ```shell
   cp .env.example .env
   ```

2. You need to run docker compose command to build the image

   ```shell
   docker compose build
   ```

3. Run the image
   ```shell
   # If both the networks are running locally
   docker compose -f docker-compose-local.yml up
   # If you are hosting on already deployed networks
   docker compose up
   ```

**IMP 🚨** : It will store all the addresses in [data/addresses.json](data/addresses.json)
