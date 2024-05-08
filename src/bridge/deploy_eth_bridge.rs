use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use scale_info::Field;
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
use crate::contract_clients::utils::{build_single_owner_account, RpcAccount};
use crate::utils::constants::{ERC20_CASM_PATH, ERC20_SIERRA_PATH};
use crate::utils::{invoke_contract, save_to_json, wait_for_transaction, JsonValueType};
use crate::CliArgs;

pub async fn deploy_eth_bridge(
    clients: &Config,
    arg_config: &CliArgs,
    core_contract: &StarknetSovereignContract,
    legacy_eth_bridge_class_hash: FieldElement,
    legacy_eth_bridge_proxy_address: FieldElement,
    eth_proxy_address: FieldElement,
    eth_erc20_class_hash: FieldElement,
    deployer_account_address: FieldElement,
    proxy_class_hash: FieldElement,
) -> Result<(StarknetLegacyEthBridge, FieldElement, FieldElement), anyhow::Error> {
    let eth_bridge = StarknetLegacyEthBridge::deploy(core_contract.client().clone()).await;

    log::debug!("Eth Bridge Deployment Successful [✅]");
    log::debug!("[🚀] Eth Bridge Address : {:?}", eth_bridge.address());
    save_to_json("ETH_l1_bridge_address", &JsonValueType::EthAddress(eth_bridge.address()))?;

    // sleeping for changing the vars in madara and rebooting
    sleep(Duration::from_secs(300)).await;

    let account = build_single_owner_account(
        clients.provider_l2(),
        &arg_config.rollup_priv_key,
        &deployer_account_address.to_string(),
        false,
    );

    let l2_bridge_address = StarknetLegacyEthBridge::deploy_l2_contracts(
        clients.provider_l2(),
        legacy_eth_bridge_class_hash,
        legacy_eth_bridge_proxy_address,
        proxy_class_hash,
        &account,
        &arg_config.rollup_priv_key,
    )
    .await;

    log::debug!("L2 Bridge Deployment Successful [✅]");
    log::debug!("[🚀] L2 Bridge Address : {:?}", l2_bridge_address);
    save_to_json("ETH_l2_bridge_address", &JsonValueType::StringType(l2_bridge_address.to_string()))?;

    let eth_address = deploy_eth_token_on_l2(
        clients.provider_l2(),
        &arg_config.rollup_priv_key,
        eth_proxy_address,
        eth_erc20_class_hash,
        &account,
        l2_bridge_address,
    )
    .await;

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
            &account.address().to_string(),
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
    private_key: &str,
    eth_proxy_address: FieldElement,
    eth_erc20_class_hash: FieldElement,
    account: &RpcAccount<'_>,
    eth_legacy_bridge_address: FieldElement,
) -> FieldElement {
    let deploy_txn = invoke_contract(
        rpc_provider_l2,
        eth_proxy_address,
        "upgrade_to",
        vec![
            eth_erc20_class_hash,
            FieldElement::ZERO,
            FieldElement::from(9u64),
            FieldElement::from_byte_slice_be("ether".as_bytes()).unwrap(),
            FieldElement::from_byte_slice_be("ETH".as_bytes()).unwrap(),
            FieldElement::from_str("18").unwrap(),
            FieldElement::from_str("10000000000000000000").unwrap(),
            FieldElement::from_str("0").unwrap(),
            account.address(),
            eth_legacy_bridge_address,
            account.address(),
            FieldElement::from_str("0").unwrap(),
            FieldElement::ONE,
        ],
        private_key,
        &account.address().to_string(),
    )
    .await;

    wait_for_transaction(rpc_provider_l2, deploy_txn.transaction_hash).await.unwrap();
    eth_proxy_address
}
