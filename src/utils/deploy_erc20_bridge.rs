use std::{str::FromStr, time::Duration};

use crate::{bridge_deploy_utils::lib::{fixtures::{madara_from, ThreadSafeMadaraClient}, utils::read_erc20_balance}, utils::{token_bridge::StarknetTokenBridge, utils::{catch_and_execute_l1_messages, invoke_contract}}};
use super::{arg_config::ArgConfig, deploy_utils::DeployClients, eth_bridge::BridgeDeployable};
use anyhow::Ok;
use ethers::types::Address;
use sp_core::{H160, U256};
use starknet_core::{types::{BlockId, BlockTag, FunctionCall}, utils::get_selector_from_name};
use starknet_ff::FieldElement;
use starknet_providers::Provider;
use tokio::time::sleep;
// use starknet_token_bridge_client::interfaces::token_bridge::StarknetTokenBridge;
use url::Url;

pub async fn deploy_erc20_bridge(deploy_clients: &DeployClients, config: ArgConfig) {
    let madara = madara_from(Url::from_str(&config.rollup_seq_url).expect("utils::deploy_eth_bridge => Error parsing the sequencer url. Please check the env vars"));

    let token_bridge = StarknetTokenBridge::deploy(deploy_clients.client().clone()).await;

    println!("Token Bridge Deployment Successful ✅");
    println!("Token Bridge Address : {:?}", token_bridge.bridge_address());
    println!("ERC 20 Token Address : {:?}", token_bridge.address());

    let l2_bridge_address = StarknetTokenBridge::deploy_l2_contracts(&madara, &config.rollup_priv_key, &config.l2_deployer_address).await;

    println!("L2 Token Bridge Deployment Successful ✅");
    println!("L2 Token Bridge Address : {:?}", l2_bridge_address);

    token_bridge.initialize(deploy_clients.address()).await;
    token_bridge.setup_l2_bridge(&madara, l2_bridge_address, &config.rollup_priv_key, &config.l2_deployer_address).await;
    token_bridge.setup_l1_bridge(H160::from_str(&config.l1_deployer_address).unwrap(), l2_bridge_address, U256::from_dec_str("100000000000000").unwrap()).await;

    catch_and_execute_l1_messages(&madara).await;
    sleep(Duration::from_millis(60000)).await;

    let l2_erc20_token_address = get_l2_token_address(&madara, &l2_bridge_address, &token_bridge.address()).await;
    println!("L2 ERC 20 Token Address : {:?}", l2_erc20_token_address);
}

pub async fn erc20_bridge_test_helper(deploy_clients: &DeployClients, config: ArgConfig) -> Result<(), anyhow::Error> {

    let madara = madara_from(Url::from_str(&config.rollup_seq_url).expect("utils::deploy_eth_bridge => Error parsing the sequencer url. Please check the env vars"));

    let token_bridge = StarknetTokenBridge::deploy(deploy_clients.client().clone()).await;

    println!("Token Bridge Deployment Successful ✅");
    println!("Token Bridge Address : {:?}", token_bridge.bridge_address());
    println!("ERC 20 Token Address : {:?}", token_bridge.address());

    let l2_bridge_address = StarknetTokenBridge::deploy_l2_contracts(&madara, &config.rollup_priv_key, &config.l2_deployer_address).await;

    println!("L2 Token Bridge Deployment Successful ✅");
    println!("L2 Token Bridge Address : {:?}", l2_bridge_address);

    token_bridge.initialize(deploy_clients.address()).await;
    token_bridge.setup_l2_bridge(&madara, l2_bridge_address, &config.rollup_priv_key, &config.l2_deployer_address).await;
    token_bridge.setup_l1_bridge(H160::from_str(&config.l1_deployer_address).unwrap(), l2_bridge_address, U256::from_dec_str("100000000000000").unwrap()).await;

    catch_and_execute_l1_messages(&madara).await;
    println!(">>>> Waiting for message to be consumed on l2");
    sleep(Duration::from_millis(60000)).await;

    let l2_erc20_token_address = get_l2_token_address(&madara, &l2_bridge_address, &token_bridge.address()).await;
    println!("L2 ERC 20 Token Address : {:?}", l2_erc20_token_address);

    token_bridge.approve(token_bridge.bridge_address(), 100000000.into()).await;
    catch_and_execute_l1_messages(&madara).await;
    println!(">>>> Approval done ✅");
    println!(">>>> Waiting for message to be consumed on l2");
    sleep(Duration::from_secs(60)).await;

    let rpc = madara.get_starknet_client().await;

    let balance_before = read_erc20_balance(&rpc,l2_erc20_token_address, FieldElement::from_str(&config.l2_deployer_address).unwrap()).await;

    token_bridge.deposit(token_bridge.address(), 10.into(), U256::from_str(&config.l2_deployer_address).unwrap(), U256::from_dec_str("100000000000000").unwrap(),).await;
    catch_and_execute_l1_messages(&madara).await;
    println!(">>>> Deposit done 💰");
    println!(">>>> Waiting for message to be consumed on l2");
    sleep(Duration::from_secs(60)).await;

    let balance_after = read_erc20_balance(&rpc,l2_erc20_token_address, FieldElement::from_str(&config.l2_deployer_address).unwrap()).await;

    assert_eq!(balance_before[0] + FieldElement::from_dec_str("10").unwrap(), balance_after[0]);

    let l1_recipient = FieldElement::from_hex_be(&config.l1_deployer_address).unwrap();

    println!(">>>> initiate_token_withdraw");
    invoke_contract(
        &madara,
        l2_bridge_address,
        "initiate_token_withdraw",
        vec![
            FieldElement::from_byte_slice_be(token_bridge.address().as_bytes()).unwrap(),
            l1_recipient,
            FieldElement::from_dec_str("5").unwrap(),
            FieldElement::ZERO,
        ],
        &config.rollup_priv_key,
        &config.l2_deployer_address
    )
    .await;

    catch_and_execute_l1_messages(&madara).await;
    println!(">>>> Waiting for message to be consumed on l2");
    sleep(Duration::from_secs(60)).await;


    let mut madara_write_lock = madara.write().await;
    madara_write_lock.create_n_blocks(2).await.expect("Unable to create empty blocks in madara");
    sleep(Duration::from_millis(12000)).await;

    let l1_recipient: Address = Address::from_str(&config.l1_deployer_address).unwrap();
    let balance_before = token_bridge.token_balance(l1_recipient).await;
    token_bridge.withdraw(token_bridge.address(), 5.into(), l1_recipient).await;
    let balance_after = token_bridge.token_balance(l1_recipient).await;

    assert_eq!(balance_before + U256::from_dec_str("5").unwrap(), balance_after);

    Ok(())

}

async fn get_l2_token_address(madara: &ThreadSafeMadaraClient, l2_bridge_address : &FieldElement, l1_erc_20_address: &H160) -> FieldElement {

    let rpc = madara.get_starknet_client().await;

    let l2_address = rpc.call(
        FunctionCall {
            contract_address: l2_bridge_address.clone(),
            entry_point_selector: get_selector_from_name("get_l2_token").unwrap(),
            calldata: vec![FieldElement::from_byte_slice_be(l1_erc_20_address.as_bytes()).unwrap()]
        },
        BlockId::Tag(BlockTag::Latest)
    ).await.unwrap()[0];

    l2_address
}