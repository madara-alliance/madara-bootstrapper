use std::str::FromStr;

use crate::{bridge_deploy_utils::lib::fixtures::madara_from, utils::token_bridge::StarknetTokenBridge};
use super::{arg_config::ArgConfig, deploy_utils::DeployClients, eth_bridge::BridgeDeployable};
use sp_core::{H160, U256};
// use starknet_token_bridge_client::interfaces::token_bridge::StarknetTokenBridge;
use url::Url;

pub async fn deploy_eth_bridge(deploy_clients: DeployClients, config: ArgConfig, deployer_address: &str) {

    let madara = madara_from(Url::from_str(&config.rollup_seq_url).expect("utils::deploy_eth_bridge => Error parsing the sequencer url. Please check the env vars"));

    let token_bridge = StarknetTokenBridge::deploy(deploy_clients.client().clone()).await;

    println!("Token Bridge Deployment Successful ✅");
    println!("Token Bridge Address : {:?}", token_bridge.address());

    let l2_bridge_address = StarknetTokenBridge::deploy_l2_contracts(&madara, &config.rollup_priv_key).await;

    println!("L2 Token Bridge Deployment Successful ✅");
    println!("L2 Token Bridge Address : {:?}", l2_bridge_address);

    token_bridge.initialize(deploy_clients.address()).await;
    token_bridge.setup_l2_bridge(&madara, l2_bridge_address).await;

    token_bridge.setup_l1_bridge(H160::from_str(deployer_address).unwrap(), l2_bridge_address, U256::from_dec_str("100000000000000").unwrap()).await;
}