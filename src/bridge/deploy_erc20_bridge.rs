use std::{str::FromStr, time::Duration};
use crate::ArgConfig;
use crate::utils::utils::invoke_contract;
use anyhow::Ok;
use ethers::types::Address;
use sp_core::{H160, U256};
use starknet_core::{types::{BlockId, BlockTag, FunctionCall}, utils::get_selector_from_name};
use starknet_ff::FieldElement;
use starknet_providers::{JsonRpcClient, Provider};
use starknet_providers::jsonrpc::HttpTransport;
use tokio::time::sleep;
use crate::bridge::helpers::deploy_utils::{Config, read_erc20_balance};
use crate::bridge::helpers::eth_bridge::BridgeDeployable;
use crate::bridge::helpers::token_bridge::StarknetTokenBridge;

pub async fn deploy_erc20_bridge(deploy_config: &Config, arg_config: ArgConfig) -> Result<(StarknetTokenBridge, FieldElement, FieldElement), anyhow::Error> {
    let token_bridge = StarknetTokenBridge::deploy(deploy_config.client().clone()).await;

    log::debug!("Token Bridge Deployment Successful ✅");
    log::debug!("Token Bridge Address : {:?}", token_bridge.bridge_address());
    log::debug!("ERC 20 Token Address : {:?}", token_bridge.address());

    let l2_bridge_address = StarknetTokenBridge::deploy_l2_contracts(&deploy_config.provider_l2(), &arg_config.rollup_priv_key, &arg_config.l2_deployer_address).await;

    log::debug!("L2 Token Bridge Deployment Successful ✅");
    log::debug!("L2 Token Bridge Address : {:?}", l2_bridge_address);

    token_bridge.initialize(deploy_config.address()).await;
    token_bridge.setup_l2_bridge(&deploy_config.provider_l2(), l2_bridge_address, &arg_config.rollup_priv_key, &arg_config.l2_deployer_address).await;
    token_bridge.setup_l1_bridge(H160::from_str(&arg_config.l1_deployer_address).unwrap(), l2_bridge_address, U256::from_dec_str("100000000000000").unwrap()).await;

    sleep(Duration::from_secs(arg_config.l1_wait_time.parse().unwrap())).await;
    sleep(Duration::from_millis(60000)).await;

    let l2_erc20_token_address = get_l2_token_address(&deploy_config.provider_l2(), &l2_bridge_address, &token_bridge.address()).await;
    log::debug!("L2 ERC 20 Token Address : {:?}", l2_erc20_token_address);

    Ok((
        token_bridge,
        l2_bridge_address,
        l2_erc20_token_address
    ))
}

pub async fn erc20_bridge_test_helper(deploy_config: &Config, arg_config: ArgConfig) -> Result<(), anyhow::Error> {
    let (token_bridge, l2_bridge_address, l2_erc20_token_address) = deploy_erc20_bridge(deploy_config, arg_config.clone()).await.expect("Error in deploying erc20 bridge");

    token_bridge.approve(token_bridge.bridge_address(), 100000000.into()).await;
    sleep(Duration::from_secs(arg_config.l1_wait_time.parse().unwrap())).await;
    log::debug!(">>>> Approval done ✅");
    log::debug!(">>>> Waiting for message to be consumed on l2");
    sleep(Duration::from_secs(60)).await;

    let balance_before = read_erc20_balance(&deploy_config.provider_l2(),l2_erc20_token_address, FieldElement::from_str(&arg_config.l2_deployer_address).unwrap()).await;

    token_bridge.deposit(token_bridge.address(), 10.into(), U256::from_str(&arg_config.l2_deployer_address).unwrap(), U256::from_dec_str("100000000000000").unwrap(),).await;
    sleep(Duration::from_secs(arg_config.l1_wait_time.parse().unwrap())).await;
    log::debug!(">>>> Deposit done 💰");
    log::debug!(">>>> Waiting for message to be consumed on l2");
    sleep(Duration::from_secs(60)).await;

    let balance_after = read_erc20_balance(&deploy_config.provider_l2(),l2_erc20_token_address, FieldElement::from_str(&arg_config.l2_deployer_address).unwrap()).await;

    assert_eq!(balance_before[0] + FieldElement::from_dec_str("10").unwrap(), balance_after[0]);

    let l1_recipient = FieldElement::from_hex_be(&arg_config.l1_deployer_address).unwrap();

    log::debug!(">>>> initiate_token_withdraw");
    invoke_contract(
        &deploy_config.provider_l2(),
        l2_bridge_address,
        "initiate_token_withdraw",
        vec![
            FieldElement::from_byte_slice_be(token_bridge.address().as_bytes()).unwrap(),
            l1_recipient,
            FieldElement::from_dec_str("5").unwrap(),
            FieldElement::ZERO,
        ],
        &arg_config.rollup_priv_key,
        &arg_config.l2_deployer_address
    )
    .await;

    sleep(Duration::from_secs(arg_config.l1_wait_time.parse().unwrap())).await;
    log::debug!(">>>> Waiting for message to be consumed on l2");
    sleep(Duration::from_secs(80)).await;
    sleep(Duration::from_secs(arg_config.l1_wait_time.parse().unwrap())).await;

    let l1_recipient: Address = Address::from_str(&arg_config.l1_deployer_address).unwrap();
    let balance_before = token_bridge.token_balance(l1_recipient).await;
    token_bridge.withdraw(token_bridge.address(), 5.into(), l1_recipient).await;
    let balance_after = token_bridge.token_balance(l1_recipient).await;

    assert_eq!(balance_before + U256::from_dec_str("5").unwrap(), balance_after);

    Ok(())
}

async fn get_l2_token_address(rpc_provider_l2: &JsonRpcClient<HttpTransport>, l2_bridge_address : &FieldElement, l1_erc_20_address: &H160) -> FieldElement {

    let l2_address = rpc_provider_l2.call(
        FunctionCall {
            contract_address: l2_bridge_address.clone(),
            entry_point_selector: get_selector_from_name("get_l2_token").unwrap(),
            calldata: vec![FieldElement::from_byte_slice_be(l1_erc_20_address.as_bytes()).unwrap()]
        },
        BlockId::Tag(BlockTag::Latest)
    ).await.unwrap()[0];

    l2_address
}