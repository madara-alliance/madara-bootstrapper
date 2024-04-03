pub mod utils;
pub mod felt;
pub mod messages;
pub mod snos;
pub mod bridge_deploy_utils;

use std::env;
use std::process;
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
    deploy_eth_bridge(&deploy_clients, config.clone()).await;
    deploy_erc20_bridge(&deploy_clients, config.clone()).await;
}