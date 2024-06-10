use std::str::FromStr;
use std::time::Duration;

use starknet_accounts::{Account, ConnectedAccount};
use starknet_ff::FieldElement;
use starknet_providers::jsonrpc::HttpTransport;
use starknet_providers::JsonRpcClient;
use tokio::time::sleep;

use crate::bridge::helpers::account_actions::{get_contract_address_from_deploy_tx, AccountActions};
use crate::contract_clients::config::Config;
use crate::contract_clients::eth_bridge::{BridgeDeployable, StarknetLegacyEthBridge};
use crate::contract_clients::starknet_sovereign::StarknetSovereignContract;
use crate::contract_clients::utils::{build_single_owner_account, RpcAccount};
use crate::utils::{convert_to_hex, invoke_contract, save_to_json, wait_for_transaction, JsonValueType};
use crate::CliArgs;

#[allow(clippy::too_many_arguments)]
pub async fn deploy_eth_bridge(
    clients: &Config,
    arg_config: &CliArgs,
    core_contract: &StarknetSovereignContract,
    legacy_eth_bridge_class_hash: FieldElement,
    legacy_eth_bridge_proxy_address: FieldElement,
    eth_proxy_address: FieldElement,
    _eth_erc20_class_hash: FieldElement,
    deployer_account_address: FieldElement,
    _proxy_class_hash: FieldElement,
    _legacy_proxy_class_hash: FieldElement,
    starkgate_proxy_class_hash: FieldElement,
    erc20_legacy_class_hash: FieldElement,
) -> Result<(StarknetLegacyEthBridge, FieldElement, FieldElement), anyhow::Error> {
    let eth_bridge = StarknetLegacyEthBridge::deploy(core_contract.client().clone()).await;

    log::debug!("Eth Bridge Deployment Successful [✅]");
    log::debug!("[🚀] Eth Bridge Address : {:?}", eth_bridge.address());
    save_to_json("ETH_l1_bridge_address", &JsonValueType::EthAddress(eth_bridge.address()))?;

    // sleeping for changing the vars in madara and rebooting
    // Not needed ig :)
    sleep(Duration::from_secs(10)).await;

    let account = build_single_owner_account(
        clients.provider_l2(),
        &arg_config.rollup_priv_key,
        &convert_to_hex(&deployer_account_address.to_string()),
        false,
    );

    let l2_bridge_address = StarknetLegacyEthBridge::deploy_l2_contracts(
        clients.provider_l2(),
        legacy_eth_bridge_class_hash,
        legacy_eth_bridge_proxy_address,
        starkgate_proxy_class_hash,
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
        erc20_legacy_class_hash,
        &account,
        l2_bridge_address,
    )
    .await;

    log::debug!("L2 ETH Token Deployment Successful [✅]");
    log::debug!("[🚀] L2 ETH Token Address : {:?}", eth_address);
    // save_to_json("l2_eth_address", &JsonValueType::StringType(eth_address.to_string()))?;

    eth_bridge.initialize(core_contract.address()).await;
    log::debug!("[🚀] ETH Bridge initialized");

    sleep(Duration::from_secs(arg_config.l1_wait_time.parse().unwrap())).await;

    eth_bridge
        .setup_l2_bridge(
            clients.provider_l2(),
            l2_bridge_address,
            eth_address,
            &arg_config.rollup_priv_key,
            &convert_to_hex(&account.address().to_string()),
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
    let deploy_tx = account
        .invoke_contract(
            account.address(),
            "deploy_contract",
            vec![eth_erc20_class_hash, FieldElement::ZERO, FieldElement::ZERO, FieldElement::ZERO],
            None,
        )
        .send()
        .await
        .expect("Error deploying the contract proxy.");
    wait_for_transaction(rpc_provider_l2, deploy_tx.transaction_hash, "deploy_eth_token_on_l2 : deploy").await.unwrap();
    let contract_address = get_contract_address_from_deploy_tx(account.provider(), &deploy_tx).await.unwrap();

    log::debug!("contract address (eth erc20) : {:?}", contract_address);

    let add_implementation_txn = invoke_contract(
        rpc_provider_l2,
        eth_proxy_address,
        "add_implementation",
        vec![
            contract_address,
            FieldElement::ZERO,
            FieldElement::from(4u64),
            FieldElement::from_byte_slice_be("Ether".as_bytes()).unwrap(),
            FieldElement::from_byte_slice_be("ETH".as_bytes()).unwrap(),
            FieldElement::from_str("18").unwrap(),
            eth_legacy_bridge_address,
            FieldElement::ZERO,
        ],
        private_key,
        &convert_to_hex(&account.address().to_string()),
    )
    .await;

    wait_for_transaction(
        rpc_provider_l2,
        add_implementation_txn.transaction_hash,
        "deploy_eth_token_on_l2 : add_implementation",
    )
    .await
    .unwrap();

    let upgrade_to_txn = invoke_contract(
        rpc_provider_l2,
        eth_proxy_address,
        "upgrade_to",
        vec![
            contract_address,
            FieldElement::ZERO,
            FieldElement::from(4u64),
            FieldElement::from_byte_slice_be("Ether".as_bytes()).unwrap(),
            FieldElement::from_byte_slice_be("ETH".as_bytes()).unwrap(),
            FieldElement::from_str("18").unwrap(),
            eth_legacy_bridge_address,
            FieldElement::ZERO,
        ],
        private_key,
        &convert_to_hex(&account.address().to_string()),
    )
    .await;

    wait_for_transaction(rpc_provider_l2, upgrade_to_txn.transaction_hash, "deploy_eth_token_on_l2 : upgrade_to")
        .await
        .unwrap();
    eth_proxy_address
}
