use std::{str::FromStr, time::Duration};

use crate::{bridge_deploy_utils::lib::{utils::read_erc20_balance}, utils::utils::{deploy_eth_token_on_l2, invoke_contract}};
use super::{arg_config::ArgConfig, deploy_utils::DeployClients, eth_bridge::{BridgeDeployable, StarknetLegacyEthBridge}};
use ethers::types::Address;
use sp_core::U256;
use starknet_ff::FieldElement;
use starknet_providers::{jsonrpc::HttpTransport, JsonRpcClient};
use tokio::time::sleep;
use url::Url;

pub async fn deploy_eth_bridge(deploy_clients: &DeployClients, config: ArgConfig) {
    let rpc_provider_l2 = JsonRpcClient::new(HttpTransport::new(Url::parse(&config.rollup_seq_url).unwrap()));

    let eth_bridge = StarknetLegacyEthBridge::deploy(deploy_clients.client().clone()).await;

    println!("Eth Bridge Deployment Successful ✅");
    println!("Eth Bridge Address : {:?}", eth_bridge.address());

    let l2_bridge_address = StarknetLegacyEthBridge::deploy_l2_contracts(&rpc_provider_l2, &config.rollup_priv_key, &config.l2_deployer_address).await;

    println!("L2 Bridge Deployment Successful ✅");
    println!("L2 Bridge Address : {:?}", l2_bridge_address);

    let l2_eth_address = deploy_eth_token_on_l2(&rpc_provider_l2, l2_bridge_address, &config.rollup_priv_key, &config.l2_deployer_address).await;

    println!("L2 ETH Token Deployment Successful ✅");
    println!("L2 ETH Token Address : {:?}", l2_eth_address);

    eth_bridge.initialize(deploy_clients.address()).await;
    println!("ETH Bridge initialized");
    eth_bridge.setup_l2_bridge(&rpc_provider_l2, l2_bridge_address, l2_eth_address, &config.rollup_priv_key, &config.l2_deployer_address).await;
    println!("ETH Bridge L2 setup complete ✅");

    eth_bridge.setup_l1_bridge("10000000000000000", "10000000000000000", l2_bridge_address).await;
    println!("ETH Bridge L1 setup complete ✅");
}

// Test helper function
// l1 ----> l2
pub async fn eth_bridge_test_helper(deploy_clients: &DeployClients, config: ArgConfig) -> Result<(), anyhow::Error> {
    let rpc_provider_l2 = JsonRpcClient::new(HttpTransport::new(Url::parse(&config.rollup_seq_url).unwrap()));

    let eth_bridge = StarknetLegacyEthBridge::deploy(deploy_clients.client().clone()).await;

    println!("Eth Bridge Deployment Successful ✅");
    println!("Eth Bridge Address : {:?}", eth_bridge.address());

    let l2_bridge_address = StarknetLegacyEthBridge::deploy_l2_contracts(&rpc_provider_l2, &config.rollup_priv_key, &config.l2_deployer_address).await;

    println!("L2 Bridge Deployment Successful ✅");
    println!("L2 Bridge Address : {:?}", l2_bridge_address);

    let l2_eth_address = deploy_eth_token_on_l2(&rpc_provider_l2, l2_bridge_address, &config.rollup_priv_key, &config.l2_deployer_address).await;

    println!("L2 ETH Token Deployment Successful ✅");
    println!("L2 ETH Token Address : {:?}", l2_eth_address);

    eth_bridge.initialize(deploy_clients.address()).await;
    println!("ETH Bridge initialized");
    eth_bridge.setup_l2_bridge(&rpc_provider_l2, l2_bridge_address, l2_eth_address, &config.rollup_priv_key, &config.l2_deployer_address).await;
    println!("ETH Bridge L2 setup complete ✅");
    
    eth_bridge.setup_l1_bridge("10000000000000000", "10000000000000000", l2_bridge_address).await;
    println!("ETH Bridge L1 setup complete ✅");


    let balance_before = read_erc20_balance(&rpc_provider_l2, l2_eth_address, FieldElement::from_hex_be(&config.l2_deployer_address.clone()).unwrap()).await;

    eth_bridge.deposit(10.into(), U256::from_str(&config.l2_deployer_address).unwrap(), 1000.into()).await;
    println!(">>>> ETH deposited on l1 💰");
    sleep(Duration::from_secs(60)).await;
    sleep(Duration::from_secs((&config.l1_wait_time).parse()?)).await;
    println!(">>>> L1 message executed on L2 //");

    let balance_after = read_erc20_balance(&rpc_provider_l2, l2_eth_address, FieldElement::from_hex_be(&config.l2_deployer_address.clone()).unwrap()).await;


    assert_eq!(balance_before[0] + FieldElement::from_dec_str("10").unwrap(), balance_after[0]);

    let l1_receipient = FieldElement::from_hex_be(&config.l1_deployer_address).unwrap();

    sleep(Duration::from_secs((&config.l1_wait_time).parse()?)).await;

    invoke_contract(&rpc_provider_l2, l2_bridge_address, "initiate_withdraw", vec![l1_receipient, FieldElement::from_dec_str("5").unwrap(), FieldElement::ZERO], &config.rollup_priv_key, &config.l2_deployer_address).await;
    println!(">>>> ETH withdrawal initiated on l2 💰");
    println!(">>>> Waiting for message to be consumed on l2");
    sleep(Duration::from_secs(60)).await;
    sleep(Duration::from_secs((&config.l1_wait_time).parse()?)).await;

    let balance_before = eth_bridge.eth_balance(Address::from_str(&config.l1_deployer_address).unwrap()).await;
    println!(">>>> eth_bridge : withdraw : init");
    eth_bridge.withdraw(5.into(), Address::from_str(&config.l1_deployer_address).unwrap()).await;
    println!(">>>> eth_bridge : withdraw : done");
    let balance_after = eth_bridge.eth_balance(Address::from_str(&config.l1_deployer_address).unwrap()).await;

    let decimals_eth = U256::from_dec_str("1000000000000000000").unwrap();

    assert_eq!(U256::checked_div(balance_before + U256::from_dec_str("5").unwrap(), decimals_eth).unwrap(), U256::checked_div(balance_after, decimals_eth).unwrap());

    Ok(())
}