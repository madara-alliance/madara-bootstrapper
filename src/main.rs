pub mod utils;
pub mod felt;
pub mod messages;
pub mod snos;
pub mod bridge_deploy_utils;

use std::env;
use std::process;
use rstest::rstest;
use utils::arg_config::ArgConfig;
use utils::deploy_erc20_bridge::deploy_erc20_bridge;
use utils::deploy_eth_bridge::deploy_eth_bridge;
use utils::deploy_utils::DeployClients;

#[tokio::main]
pub async fn main() {

    // Reqs : 
    // ----
    // - Args :
    //      - eth_rpc
    //      - eth_priv_key
    //      - rollup_sequencer_url
    //      - rollup_priv_key
    //      - L1 deployer address : will be used as a governor in bridge contracts

    let args: Vec<String> = env::args().collect();

    // args config
    let config = ArgConfig::new(&args).unwrap_or_else(|err| {
        eprintln!("Problem parsing args : {}", err);
        process::exit(1)
    });

   deploy_bridges(config).await;
}

async fn deploy_bridges(config: ArgConfig) {
    let deploy_clients = DeployClients::deploy(&config).await;
    println!(">>>>> core address : {:?}", deploy_clients.address());
    deploy_clients.initialize_for_goerli(0u64.into(), 0u64.into()).await;
    deploy_eth_bridge(&deploy_clients, config.clone()).await;
    deploy_erc20_bridge(&deploy_clients, config.clone()).await;
}

#[rstest]
#[tokio::test]
async fn deploy_bridge() -> Result<(), anyhow::Error> {
    use bridge_deploy_utils::lib::constants::CAIRO_1_ACCOUNT_CONTRACT;


    const ETH_RPC: &str = "http://127.0.0.1:8545";
    const ETH_PRIV_KEY: &str = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    const ROLLUP_SEQ_URL: &str = "http://127.0.0.1:9944";
    const ROLLUP_PRIV_KEY: &str = "";
    const ETH_CHAIN_ID: &str = "31337";
    const L1_DEPLOYER_ADDRESS: &str = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266";
    const L2_DEPLOYER_ADDRESS: &str = CAIRO_1_ACCOUNT_CONTRACT;

    let args: Vec<String> = vec![String::from("temp"), String::from(ETH_RPC), String::from(ETH_PRIV_KEY), String::from(ROLLUP_SEQ_URL), String::from(ROLLUP_PRIV_KEY), String::from(ETH_CHAIN_ID), String::from(L1_DEPLOYER_ADDRESS), String::from(L2_DEPLOYER_ADDRESS)];

    let config = ArgConfig::new(&args).unwrap_or_else(|err| {
        eprintln!("Problem parsing args : {}", err);
        process::exit(1)
    });
    
    deploy_bridges(config).await;

    Ok(())
}

#[rstest]
#[tokio::test]
async fn deposit_and_withdraw_eth_bridge() -> Result<(), anyhow::Error> {
    use bridge_deploy_utils::lib::constants::CAIRO_1_ACCOUNT_CONTRACT;
    use utils::deploy_eth_bridge::eth_bridge_test_helper;
    
    const ETH_RPC: &str = "http://127.0.0.1:8545";
    const ETH_PRIV_KEY: &str = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    const ROLLUP_SEQ_URL: &str = "http://127.0.0.1:9944";
    // const ROLLUP_PRIV_KEY: &str = "0x00c1cf1490de1352865301bb8705143f3ef938f97fdf892f1090dcb5ac7bcd1d";
    const ROLLUP_PRIV_KEY: &str = "";
    const ETH_CHAIN_ID: &str = "31337";
    const L1_DEPLOYER_ADDRESS: &str = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266";
    const L2_DEPLOYER_ADDRESS: &str = CAIRO_1_ACCOUNT_CONTRACT;

    let args: Vec<String> = vec![String::from("temp"), String::from(ETH_RPC), String::from(ETH_PRIV_KEY), String::from(ROLLUP_SEQ_URL), String::from(ROLLUP_PRIV_KEY), String::from(ETH_CHAIN_ID), String::from(L1_DEPLOYER_ADDRESS), String::from(L2_DEPLOYER_ADDRESS)];

    let config = ArgConfig::new(&args).unwrap_or_else(|err| {
        eprintln!("Problem parsing args : {}", err);
        process::exit(1)
    });
    
    let deploy_clients = DeployClients::deploy(&config).await;
    println!(">>>>> core address : {:?}", deploy_clients.address());
    deploy_clients.initialize_for_goerli(0u64.into(), 0u64.into()).await;

    let _ = eth_bridge_test_helper(&deploy_clients, config).await;
    
    Ok(())
}

#[rstest]
#[tokio::test]
async fn deposit_and_withdraw_erc20_bridge() -> Result<(), anyhow::Error> {
    use bridge_deploy_utils::lib::constants::CAIRO_1_ACCOUNT_CONTRACT;
    use utils::deploy_erc20_bridge::erc20_bridge_test_helper;
    
    const ETH_RPC: &str = "http://127.0.0.1:8545";
    const ETH_PRIV_KEY: &str = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    const ROLLUP_SEQ_URL: &str = "http://127.0.0.1:9944";
    // const ROLLUP_PRIV_KEY: &str = "0x00c1cf1490de1352865301bb8705143f3ef938f97fdf892f1090dcb5ac7bcd1d";
    const ROLLUP_PRIV_KEY: &str = "";
    const ETH_CHAIN_ID: &str = "31337";
    const L1_DEPLOYER_ADDRESS: &str = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266";
    const L2_DEPLOYER_ADDRESS: &str = CAIRO_1_ACCOUNT_CONTRACT;

    let args: Vec<String> = vec![String::from("temp"), String::from(ETH_RPC), String::from(ETH_PRIV_KEY), String::from(ROLLUP_SEQ_URL), String::from(ROLLUP_PRIV_KEY), String::from(ETH_CHAIN_ID), String::from(L1_DEPLOYER_ADDRESS), String::from(L2_DEPLOYER_ADDRESS)];

    let config = ArgConfig::new(&args).unwrap_or_else(|err| {
        eprintln!("Problem parsing args : {}", err);
        process::exit(1)
    });
    
    let deploy_clients = DeployClients::deploy(&config).await;
    println!(">>>>> core address : {:?}", deploy_clients.address());
    deploy_clients.initialize_for_goerli(0u64.into(), 0u64.into()).await;

    let _ = erc20_bridge_test_helper(&deploy_clients, config).await;
    
    Ok(())
}