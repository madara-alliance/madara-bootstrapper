pub mod bridge;
#[cfg(test)]
pub mod tests;
pub mod utils;

use crate::bridge::contract_clients::config::{get_bridge_init_configs, Config};
use crate::bridge::contract_clients::starknet_sovereign::StarknetSovereignContract;
use crate::bridge::deploy_erc20_bridge::deploy_erc20_bridge;
use crate::bridge::deploy_eth_bridge::deploy_eth_bridge;
use clap::Parser;
use dotenv::dotenv;
use std::process;
use utils::arg_config::ArgConfig;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    #[clap(long, env, default_value = "http://127.0.0.1:8545")]
    eth_rpc: String,
    #[clap(
        long,
        env,
        default_value = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
    )]
    eth_priv_key: String,
    #[clap(long, env, default_value = "http://127.0.0.1:9944")]
    rollup_seq_url: String,
    #[clap(long, env, default_value = "")]
    rollup_priv_key: String,
    #[clap(long, env, default_value_t = 31337)]
    eth_chain_id: u64,
    #[clap(
        long,
        env,
        default_value = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
    )]
    l1_deployer_address: String,
    #[clap(
        long,
        env,
        default_value = "0x0000000000000000000000000000000000000000000000000000000000000004"
    )]
    l2_deployer_address: String,
    #[clap(long, env, default_value = "15")]
    l1_wait_time: String,
    #[clap(
        long,
        env,
        default_value = "0x41fc2a467ef8649580631912517edcab7674173f1dbfa2e9b64fbcd82bc4d79"
    )]
    sn_os_program_hash: String,
    #[clap(long, env, default_value = "StarknetOsConfig1")]
    config_hash_version: String,
    #[clap(long, env, default_value = "MADARA")]
    app_chain_id: String,
    #[clap(
        long,
        env,
        default_value = "0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7"
    )]
    fee_token_address: String,
}

#[tokio::main]
pub async fn main() {
    env_logger::init();
    dotenv().ok();

    let args = CliArgs::parse();

    // log::debug!("env : {:?}", args);

    // args config
    let config = ArgConfig::new(&args).unwrap_or_else(|err| {
        log::error!("Problem parsing args ❌ : {}", err);
        process::exit(1)
    });

    deploy_bridges(&config).await;
}

pub async fn deploy_bridges(config: &ArgConfig) {
    let clients = Config::init(&config).await;
    let core_contract_client = StarknetSovereignContract::deploy(&clients).await;
    log::debug!("core address [📦] : {:?}", core_contract_client.address());
    let (program_hash, config_hash) = get_bridge_init_configs(&config);
    core_contract_client
        .initialize_for_goerli(0u64.into(), 0u64.into(), program_hash, config_hash)
        .await;
    log::debug!("bridge init for goerli successful [✅]");
    log::debug!(">>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> [ETH BRIDGE] ⏳");
    deploy_eth_bridge(&clients, config.clone(), &core_contract_client)
        .await
        .expect("Error in deploying ETH bridge");
    log::debug!("ETH BRIDGE DEPLOYED [✅]");
    log::debug!(">>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>[ERC20 BRIDGE] ⏳");
    deploy_erc20_bridge(&clients, config.clone(), &core_contract_client)
        .await
        .expect("Error in deploying ERC20 bridge");
    log::debug!("ERC20 BRIDGE DEPLOYED [✅]");
}
