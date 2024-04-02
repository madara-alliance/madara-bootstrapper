use std::str::FromStr;

use crate::{bridge_deploy_utils::lib::fixtures::madara_from, utils::utils::deploy_eth_token_on_l2};
use super::{arg_config::ArgConfig, deploy_utils::DeployClients, eth_bridge::{BridgeDeployable, StarknetLegacyEthBridge}};
use url::Url;

pub async fn deploy_eth_bridge(deploy_clients: DeployClients, config: ArgConfig) {

    let madara = madara_from(Url::from_str(&config.rollup_seq_url).expect("utils::deploy_eth_bridge => Error parsing the sequencer url. Please check the env vars"));

    let eth_bridge = StarknetLegacyEthBridge::deploy(deploy_clients.client().clone()).await;

    println!("Eth Bridge Deployment Successful ✅");
    println!("Eth Bridge Address : {:?}", eth_bridge.address());

    let l2_bridge_address = StarknetLegacyEthBridge::deploy_l2_contracts(&madara, &config.rollup_priv_key).await;

    println!("L2 Bridge Deployment Successful ✅");
    println!("L2 Bridge Address : {:?}", l2_bridge_address);

    let l2_eth_address = deploy_eth_token_on_l2(&madara, l2_bridge_address, &config.rollup_priv_key).await;

    println!("L2 ETH Token Deployment Successful ✅");
    println!("L2 ETH Token Address : {:?}", l2_eth_address);

    eth_bridge.initialize(deploy_clients.address()).await;
    println!("ETH Bridge initialized");
    eth_bridge.setup_l2_bridge(&madara, l2_bridge_address, l2_eth_address).await;
    println!("ETH Bridge L2 setup complete ✅");
    eth_bridge.setup_l1_bridge("10000000000000000", "10000000000000000", l2_bridge_address).await;
    println!("ETH Bridge L1 setup complete ✅");
}