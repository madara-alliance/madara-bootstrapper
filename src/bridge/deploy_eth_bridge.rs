use std::{str::FromStr, time::Duration};
use std::sync::Arc;
use ethers::types::Address;
use sp_core::U256;
use starknet_accounts::Account;
use starknet_contract::ContractFactory;
use starknet_ff::FieldElement;
use starknet_providers::{jsonrpc::HttpTransport, JsonRpcClient};
use tokio::time::sleep;
use crate::bridge::helpers::account_actions::{AccountActions, get_contract_address_from_deploy_tx};
use crate::bridge::helpers::deploy_utils::{build_single_owner_account, Config, read_erc20_balance};
use crate::bridge::helpers::eth_bridge::{BridgeDeployable, StarknetLegacyEthBridge};
use crate::utils::arg_config::ArgConfig;
use crate::utils::constants::{ERC20_CASM_PATH, ERC20_SIERRA_PATH};
use crate::utils::utils::{invoke_contract, wait_for_transaction};

pub async fn deploy_eth_bridge(deploy_config: &Config, arg_config: ArgConfig) -> Result<(StarknetLegacyEthBridge, FieldElement, FieldElement), anyhow::Error> {
    let eth_bridge = StarknetLegacyEthBridge::deploy(deploy_config.client().clone()).await;

    log::debug!("Eth Bridge Deployment Successful ✅");
    log::debug!("Eth Bridge Address : {:?}", eth_bridge.address());

    let l2_bridge_address = StarknetLegacyEthBridge::deploy_l2_contracts(&deploy_config.provider_l2(), &arg_config.rollup_priv_key, &arg_config.l2_deployer_address).await;

    log::debug!("L2 Bridge Deployment Successful ✅");
    log::debug!("L2 Bridge Address : {:?}", l2_bridge_address);

    let l2_eth_address = deploy_eth_token_on_l2(&deploy_config.provider_l2(), l2_bridge_address, &arg_config.rollup_priv_key, &arg_config.l2_deployer_address).await;

    log::debug!("L2 ETH Token Deployment Successful ✅");
    log::debug!("L2 ETH Token Address : {:?}", l2_eth_address);

    eth_bridge.initialize(deploy_config.address()).await;
    log::debug!("ETH Bridge initialized");
    eth_bridge.setup_l2_bridge(&deploy_config.provider_l2(), l2_bridge_address, l2_eth_address, &arg_config.rollup_priv_key, &arg_config.l2_deployer_address).await;
    log::debug!("ETH Bridge L2 setup complete ✅");

    eth_bridge.setup_l1_bridge("10000000000000000", "10000000000000000", l2_bridge_address).await;
    log::debug!("ETH Bridge L1 setup complete ✅");

    Ok((
        eth_bridge,
        l2_bridge_address,
        l2_eth_address
    ))
}

// Test helper function
// l1 ----> l2
pub async fn eth_bridge_test_helper(deploy_config: &Config, arg_config: ArgConfig) -> Result<(), anyhow::Error> {
    let (eth_bridge, l2_bridge_address, l2_eth_address) = deploy_eth_bridge(deploy_config, arg_config.clone()).await.expect("Error in deploying eth bridge");

    let balance_before = read_erc20_balance(&deploy_config.provider_l2(), l2_eth_address, FieldElement::from_hex_be(&arg_config.l2_deployer_address.clone()).unwrap()).await;

    eth_bridge.deposit(10.into(), U256::from_str(&arg_config.l2_deployer_address).unwrap(), 1000.into()).await;
    log::debug!(">>>> ETH deposited on l1 💰");
    sleep(Duration::from_secs(60)).await;
    sleep(Duration::from_secs((&arg_config.l1_wait_time).parse()?)).await;
    log::debug!(">>>> L1 message executed on L2 //");

    let balance_after = read_erc20_balance(&deploy_config.provider_l2(), l2_eth_address, FieldElement::from_hex_be(&arg_config.l2_deployer_address.clone()).unwrap()).await;


    assert_eq!(balance_before[0] + FieldElement::from_dec_str("10").unwrap(), balance_after[0]);

    let l1_receipient = FieldElement::from_hex_be(&arg_config.l1_deployer_address).unwrap();

    sleep(Duration::from_secs((&arg_config.l1_wait_time).parse()?)).await;

    invoke_contract(&deploy_config.provider_l2(), l2_bridge_address, "initiate_withdraw", vec![l1_receipient, FieldElement::from_dec_str("5").unwrap(), FieldElement::ZERO], &arg_config.rollup_priv_key, &arg_config.l2_deployer_address).await;
    log::debug!(">>>> ETH withdrawal initiated on l2 💰");
    log::debug!(">>>> Waiting for message to be consumed on l2");
    sleep(Duration::from_secs(60)).await;
    sleep(Duration::from_secs((&arg_config.l1_wait_time).parse()?)).await;

    let balance_before = eth_bridge.eth_balance(Address::from_str(&arg_config.l1_deployer_address).unwrap()).await;
    log::debug!(">>>> eth_bridge : withdraw : init");
    eth_bridge.withdraw(5.into(), Address::from_str(&arg_config.l1_deployer_address).unwrap()).await;
    log::debug!(">>>> eth_bridge : withdraw : done");
    let balance_after = eth_bridge.eth_balance(Address::from_str(&arg_config.l1_deployer_address).unwrap()).await;

    let decimals_eth = U256::from_dec_str("1000000000000000000").unwrap();

    assert_eq!(U256::checked_div(balance_before + U256::from_dec_str("5").unwrap(), decimals_eth).unwrap(), U256::checked_div(balance_after, decimals_eth).unwrap());

    Ok(())
}

pub async fn deploy_eth_token_on_l2(rpc_provider_l2: &JsonRpcClient<HttpTransport>, minter: FieldElement, private_key: &str, address: &str) -> FieldElement {
    let account = build_single_owner_account(&rpc_provider_l2, private_key, address, false);

    let (class_hash, contract_artifact) = account.declare_contract_params_sierra(ERC20_SIERRA_PATH, ERC20_CASM_PATH);
    let flattened_class = contract_artifact.clone().flatten().unwrap();

    let declare_txn = account.declare(Arc::new(flattened_class), class_hash).send().await.expect("Unable to declare ERC20 token on L2");
    let sierra_class_hash = contract_artifact.class_hash().unwrap();

    wait_for_transaction(rpc_provider_l2, declare_txn.transaction_hash).await.unwrap();

    let contract_factory = ContractFactory::new(sierra_class_hash, account.clone());

    let deploy_tx = &contract_factory.deploy(
        vec![
            FieldElement::from_byte_slice_be("ether".as_bytes()).unwrap(), // Name
            FieldElement::from_byte_slice_be("ETH".as_bytes()).unwrap(),   // Symbol
            FieldElement::from_str("18").unwrap(),                         // Decimals
            FieldElement::from_str("10000000000000000000").unwrap(),       // Initial supply low
            FieldElement::from_str("0").unwrap(),                          // Initial supply high
            account.address(),                                             // recipient
            minter,                                                        // permitted_minter
            account.address(),                                             // provisional_governance_admin
            FieldElement::from_str("0").unwrap(),                          // upgrade_delay
        ],
        FieldElement::ZERO,
        true,
    ).send().await.expect("Unable to deploy ERC20 token on L2");

    wait_for_transaction(rpc_provider_l2, declare_txn.transaction_hash).await.unwrap();

    let address = get_contract_address_from_deploy_tx(&rpc_provider_l2, deploy_tx).await.expect("Error getting contract address from transaction hash");

    address
}