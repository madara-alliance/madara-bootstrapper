use crate::bridge::contract_clients::config::{read_erc20_balance, Config};
use crate::bridge::contract_clients::starknet_sovereign::StarknetSovereignContract;
use crate::bridge::deploy_erc20_bridge::deploy_erc20_bridge;
use crate::bridge::deploy_eth_bridge::deploy_eth_bridge;
use crate::utils::arg_config::ArgConfig;
use crate::utils::utils::invoke_contract;
use ethers::addressbook::Address;
use ethers::prelude::U256;
use starknet_ff::FieldElement;
use std::str::FromStr;
use std::time::Duration;
use tokio::time::sleep;

pub async fn eth_bridge_test_helper(
    clients: &Config,
    arg_config: &ArgConfig,
    core_contract: &StarknetSovereignContract,
) -> Result<(), anyhow::Error> {
    let (eth_bridge, l2_bridge_address, l2_eth_address) =
        deploy_eth_bridge(clients, arg_config.clone(), core_contract)
            .await
            .expect("Error in deploying eth bridge [❌]");

    let balance_before = read_erc20_balance(
        &clients.provider_l2(),
        l2_eth_address,
        FieldElement::from_hex_be(&arg_config.l2_deployer_address.clone()).unwrap(),
    )
    .await;

    eth_bridge
        .deposit(
            10.into(),
            U256::from_str(&arg_config.l2_deployer_address).unwrap(),
            1000.into(),
        )
        .await;
    log::debug!("ETH deposited on l1 [💰]");
    sleep(Duration::from_secs(60)).await;
    sleep(Duration::from_secs((&arg_config.l1_wait_time).parse()?)).await;
    log::debug!("L1 message executed on L2 [🔁]");

    let balance_after = read_erc20_balance(
        &clients.provider_l2(),
        l2_eth_address,
        FieldElement::from_hex_be(&arg_config.l2_deployer_address.clone()).unwrap(),
    )
    .await;

    assert_eq!(
        balance_before[0] + FieldElement::from_dec_str("10").unwrap(),
        balance_after[0]
    );

    let l1_receipient = FieldElement::from_hex_be(&arg_config.l1_deployer_address).unwrap();

    sleep(Duration::from_secs((&arg_config.l1_wait_time).parse()?)).await;

    invoke_contract(
        &clients.provider_l2(),
        l2_bridge_address,
        "initiate_withdraw",
        vec![
            l1_receipient,
            FieldElement::from_dec_str("5").unwrap(),
            FieldElement::ZERO,
        ],
        &arg_config.rollup_priv_key,
        &arg_config.l2_deployer_address,
    )
    .await;
    log::debug!("ETH withdrawal initiated on l2 [💰]");
    log::debug!("Waiting for message to be consumed on l2 [⏳]");
    sleep(Duration::from_secs(60)).await;
    sleep(Duration::from_secs((&arg_config.l1_wait_time).parse()?)).await;

    let balance_before = eth_bridge
        .eth_balance(Address::from_str(&arg_config.l1_deployer_address).unwrap())
        .await;
    log::debug!("Withdraw initiated on ETH Bridge [⏳]");
    eth_bridge
        .withdraw(
            5.into(),
            Address::from_str(&arg_config.l1_deployer_address).unwrap(),
        )
        .await;
    log::debug!("Withdraw completed on ETH Bridge [✅]");
    let balance_after = eth_bridge
        .eth_balance(Address::from_str(&arg_config.l1_deployer_address).unwrap())
        .await;

    let decimals_eth = U256::from_dec_str("1000000000000000000").unwrap();

    assert_eq!(
        U256::checked_div(
            balance_before + U256::from_dec_str("5").unwrap(),
            decimals_eth
        )
        .unwrap(),
        U256::checked_div(balance_after, decimals_eth).unwrap()
    );

    Ok(())
}

pub async fn erc20_bridge_test_helper(
    clients: &Config,
    arg_config: &ArgConfig,
    core_contract: &StarknetSovereignContract,
) -> Result<(), anyhow::Error> {
    let (token_bridge, l2_bridge_address, l2_erc20_token_address) =
        deploy_erc20_bridge(clients, arg_config.clone(), core_contract)
            .await
            .expect("Error in deploying erc20 bridge [❌]");

    token_bridge
        .approve(token_bridge.bridge_address(), 100000000.into())
        .await;
    sleep(Duration::from_secs(
        arg_config.l1_wait_time.parse().unwrap(),
    ))
    .await;
    log::debug!("Approval done [✅]");
    log::debug!("Waiting for message to be consumed on l2 [⏳]");
    sleep(Duration::from_secs(60)).await;

    let balance_before = read_erc20_balance(
        &clients.provider_l2(),
        l2_erc20_token_address,
        FieldElement::from_str(&arg_config.l2_deployer_address).unwrap(),
    )
    .await;

    token_bridge
        .deposit(
            token_bridge.address(),
            10.into(),
            U256::from_str(&arg_config.l2_deployer_address).unwrap(),
            U256::from_dec_str("100000000000000").unwrap(),
        )
        .await;
    sleep(Duration::from_secs(
        arg_config.l1_wait_time.parse().unwrap(),
    ))
    .await;
    log::debug!("Deposit done [💰]");
    log::debug!("Waiting for message to be consumed on l2 [⏳]");
    sleep(Duration::from_secs(60)).await;

    let balance_after = read_erc20_balance(
        &clients.provider_l2(),
        l2_erc20_token_address,
        FieldElement::from_str(&arg_config.l2_deployer_address).unwrap(),
    )
    .await;

    assert_eq!(
        balance_before[0] + FieldElement::from_dec_str("10").unwrap(),
        balance_after[0]
    );

    let l1_recipient = FieldElement::from_hex_be(&arg_config.l1_deployer_address).unwrap();

    log::debug!("Initiated token withdraw on L2 [⏳]");
    invoke_contract(
        &clients.provider_l2(),
        l2_bridge_address,
        "initiate_token_withdraw",
        vec![
            FieldElement::from_byte_slice_be(token_bridge.address().as_bytes()).unwrap(),
            l1_recipient,
            FieldElement::from_dec_str("5").unwrap(),
            FieldElement::ZERO,
        ],
        &arg_config.rollup_priv_key,
        &arg_config.l2_deployer_address,
    )
    .await;

    sleep(Duration::from_secs(
        arg_config.l1_wait_time.parse().unwrap(),
    ))
    .await;
    log::debug!("Waiting for message to be consumed on l2 [⏳]");
    sleep(Duration::from_secs(80)).await;
    sleep(Duration::from_secs(
        arg_config.l1_wait_time.parse().unwrap(),
    ))
    .await;

    let l1_recipient: Address = Address::from_str(&arg_config.l1_deployer_address).unwrap();
    let balance_before = token_bridge.token_balance(l1_recipient).await;
    token_bridge
        .withdraw(token_bridge.address(), 5.into(), l1_recipient)
        .await;
    let balance_after = token_bridge.token_balance(l1_recipient).await;

    assert_eq!(
        balance_before + U256::from_dec_str("5").unwrap(),
        balance_after
    );

    log::debug!("Token withdraw successful [✅]");

    anyhow::Ok(())
}
