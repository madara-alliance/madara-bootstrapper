pub mod utils;
pub mod felt;
pub mod bridge;

use std::process;
use clap::Parser;
use rstest::rstest;
use utils::arg_config::ArgConfig;
use crate::bridge::deploy_erc20_bridge::deploy_erc20_bridge;
use crate::bridge::deploy_eth_bridge::deploy_eth_bridge;
use crate::bridge::helpers::deploy_utils::Config;


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    #[arg(long, default_value = "http://127.0.0.1:8545")]
    eth_rpc: String,
    #[arg(long, default_value = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80")]
    eth_priv_key: String,
    #[arg(long, default_value = "http://127.0.0.1:9944")]
    rollup_seq_url: String,
    #[arg(long, default_value = "")]
    rollup_priv_key: String,
    #[arg(long, default_value_t = 31337)]
    eth_chain_id: u64,
    #[arg(long, default_value = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266")]
    l1_deployer_address: String,
    #[arg(long, default_value = "0x0000000000000000000000000000000000000000000000000000000000000004")]
    l2_deployer_address: String,
    #[arg(long, default_value = "15")]
    l1_wait_time: String,
}

#[tokio::main]
pub async fn main() {
    let args = CliArgs::parse();
    println!("{:?}", &args);

    // args config
    let config = ArgConfig::new(&args).unwrap_or_else(|err| {
        log::error!("Problem parsing args : {}", err);
        process::exit(1)
    });

   deploy_bridges(config).await;
}

async fn deploy_bridges(config: ArgConfig) {
    let deploy_clients = Config::deploy(&config).await;
    log::info!("core address : {:?}", deploy_clients.address());
    deploy_clients.initialize_for_goerli(0u64.into(), 0u64.into()).await;
    log::info!("bridge init for goerli");
    log::info!(">>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>[ETH BRIDGE]");
    deploy_eth_bridge(&deploy_clients, config.clone()).await.expect("Error in deploying ETH bridge");
    log::info!("ETH BRIDGE DEPLOYED");
    log::info!(">>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>[ERC20 BRIDGE]");
    deploy_erc20_bridge(&deploy_clients, config.clone()).await.expect("Error in deploying ERC20 bridge");
    log::info!("ERC20 BRIDGE DEPLOYED");
}

#[rstest]
#[tokio::test]
async fn deploy_bridge() -> Result<(), anyhow::Error> {
    let args = CliArgs::parse();

    let config = ArgConfig::new(&args).unwrap_or_else(|err| {
        log::error!("Problem parsing args : {}", err);
        process::exit(1)
    });
    
    deploy_bridges(config).await;

    Ok(())
}

#[rstest]
#[tokio::test]
async fn deposit_and_withdraw_eth_bridge() -> Result<(), anyhow::Error> {
    use crate::bridge::deploy_eth_bridge::eth_bridge_test_helper;

    let args = CliArgs::parse();

    let config = ArgConfig::new(&args).unwrap_or_else(|err| {
        log::error!("Problem parsing args : {}", err);
        process::exit(1)
    });
    
    let deploy_clients = Config::deploy(&config).await;
    log::debug!("core address : {:?}", deploy_clients.address());
    deploy_clients.initialize_for_goerli(0u64.into(), 0u64.into()).await;
    log::trace!("bridge init for goerli");

    let _ = eth_bridge_test_helper(&deploy_clients, config).await;
    
    Ok(())
}

#[rstest]
#[tokio::test]
async fn deposit_and_withdraw_erc20_bridge() -> Result<(), anyhow::Error> {
    use crate::bridge::deploy_erc20_bridge::erc20_bridge_test_helper;

    let args = CliArgs::parse();

    let config = ArgConfig::new(&args).unwrap_or_else(|err| {
        log::error!("Problem parsing args : {}", err);
        process::exit(1)
    });
    
    let deploy_clients = Config::deploy(&config).await;
    log::debug!("core address : {:?}", deploy_clients.address());
    deploy_clients.initialize_for_goerli(0u64.into(), 0u64.into()).await;
    log::trace!("bridge init for goerli");

    let _ = erc20_bridge_test_helper(&deploy_clients, config).await;
    
    Ok(())
}
