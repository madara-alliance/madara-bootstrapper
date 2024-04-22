use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use starknet_accounts::Account;
use starknet_contract::ContractFactory;
use starknet_ff::FieldElement;
use starknet_providers::jsonrpc::HttpTransport;
use starknet_providers::JsonRpcClient;
use tokio::time::sleep;

use crate::bridge::helpers::account_actions::AccountActions;
use crate::contract_clients::config::Config;
use crate::contract_clients::eth_bridge::{BridgeDeployable, StarknetLegacyEthBridge};
use crate::contract_clients::starknet_sovereign::StarknetSovereignContract;
use crate::contract_clients::utils::build_single_owner_account;
use crate::utils::constants::{ERC20_CASM_PATH, ERC20_SIERRA_PATH};
use crate::utils::{save_to_json, wait_for_transaction, JsonValueType};
use crate::CliArgs;

pub async fn deploy_eth_bridge(
    clients: &Config,
    arg_config: &CliArgs,
    core_contract: &StarknetSovereignContract,
) -> Result<(StarknetLegacyEthBridge, FieldElement, FieldElement), anyhow::Error> {
    let eth_bridge = StarknetLegacyEthBridge::deploy(core_contract.client().clone()).await;

    log::debug!("Eth Bridge Deployment Successful [✅]");
    log::debug!("[🚀] Eth Bridge Address : {:?}", eth_bridge.address());
    save_to_json("ETH_l1_bridge_address", &JsonValueType::EthAddress(eth_bridge.address()))?;

    let l2_bridge_address = StarknetLegacyEthBridge::deploy_l2_contracts(
        clients.provider_l2(),
        &arg_config.rollup_priv_key,
        &arg_config.l2_deployer_address,
    )
    .await;

    log::debug!("L2 Bridge Deployment Successful [✅]");
    log::debug!("[🚀] L2 Bridge Address : {:?}", l2_bridge_address);
    save_to_json("ETH_l2_bridge_address", &JsonValueType::StringType(l2_bridge_address.to_string()))?;

    let eth_address =
        FieldElement::from_str("0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7").unwrap();

    // let eth_address = deploy_eth_token_on_l2(
    //     clients.provider_l2(),
    //     l2_bridge_address,
    //     &arg_config.rollup_priv_key,
    //     &arg_config.l2_deployer_address,
    // )
    //     .await;

    log::debug!("L2 ETH Token Deployment Successful [✅]");
    log::debug!("[🚀] L2 ETH Token Address : {:?}", eth_address);
    save_to_json("l2_eth_address", &JsonValueType::StringType(eth_address.to_string()))?;

    eth_bridge.initialize(core_contract.address()).await;
    log::debug!("[🚀] ETH Bridge initialized");

    sleep(Duration::from_secs(arg_config.l1_wait_time.parse().unwrap())).await;

    eth_bridge
        .setup_l2_bridge(
            clients.provider_l2(),
            l2_bridge_address,
            eth_address,
            &arg_config.rollup_priv_key,
            &arg_config.l2_deployer_address,
        )
        .await;
    log::debug!("ETH Bridge L2 setup complete [✅]");

    eth_bridge
        .setup_l1_bridge(
            "10000000000000000000000000000000000000000",
            "10000000000000000000000000000000000000000",
            l2_bridge_address,
        )
        .await;
    log::debug!("ETH Bridge L1 setup complete [✅]");

    Ok((eth_bridge, l2_bridge_address, eth_address))
}

pub async fn deploy_eth_token_on_l2(
    rpc_provider_l2: &JsonRpcClient<HttpTransport>,
    minter: FieldElement,
    private_key: &str,
    address: &str,
) -> FieldElement {
    let account = build_single_owner_account(rpc_provider_l2, private_key, address, false);

    let (class_hash, contract_artifact) = account.declare_contract_params_sierra(ERC20_SIERRA_PATH, ERC20_CASM_PATH);
    let flattened_class = contract_artifact.clone().flatten().unwrap();

    let declare_txn = account
        .declare(Arc::new(flattened_class), class_hash)
        .send()
        .await
        .expect("Unable to declare ERC20 token on L2");
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
    );

    let deployed_address = deploy_tx.deployed_address();
    deploy_tx.send().await.expect("[❌] Unable to deploy ERC20 token on L2");

    deployed_address
}
