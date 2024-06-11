use std::str::FromStr;
use std::time::Duration;

use starknet_ff::FieldElement;
use tokio::time::sleep;

use crate::contract_clients::init_state::{
    declare_contract_util_func, deploy_proxy_contract, init_governance_proxy, DeclarationInput,
};
use crate::contract_clients::utils::RpcAccount;
use crate::utils::constants::{ERC20_LEGACY_PATH, LEGACY_BRIDGE_PATH, PROXY_LEGACY_PATH, STARKGATE_PROXY_PATH};
use crate::utils::{save_to_json, JsonValueType};
use crate::CliArgs;

pub struct EthBridgeInitOutput {
    pub legacy_proxy_class_hash: FieldElement,
    pub starkgate_proxy_class_hash: FieldElement,
    pub erc20_legacy_class_hash: FieldElement,
    pub legacy_eth_bridge_class_hash: FieldElement,
    pub eth_proxy_address: FieldElement,
    pub eth_bridge_proxy_address: FieldElement,
}

pub async fn eth_bridge_init_func(
    arg_config: &CliArgs,
    user_account: RpcAccount<'_>,
    account_address: FieldElement,
) -> EthBridgeInitOutput {
    let legacy_proxy_class_hash = declare_contract_util_func(DeclarationInput::LegacyDeclarationInputs(
        String::from(PROXY_LEGACY_PATH),
        arg_config.rollup_seq_url.clone(),
    ))
    .await;
    log::debug!("Legacy Proxy Class Hash Declared !!!");
    save_to_json("legacy_proxy_class_hash", &JsonValueType::StringType(legacy_proxy_class_hash.to_string())).unwrap();
    sleep(Duration::from_secs(10)).await;

    let starkgate_proxy_class_hash = declare_contract_util_func(DeclarationInput::LegacyDeclarationInputs(
        String::from(STARKGATE_PROXY_PATH),
        arg_config.rollup_seq_url.clone(),
    ))
    .await;
    log::debug!("Starkgate Proxy Class Hash Declared !!!");
    save_to_json("starkgate_proxy_class_hash", &JsonValueType::StringType(starkgate_proxy_class_hash.to_string()))
        .unwrap();
    sleep(Duration::from_secs(10)).await;

    let erc20_legacy_class_hash = declare_contract_util_func(DeclarationInput::LegacyDeclarationInputs(
        String::from(ERC20_LEGACY_PATH),
        arg_config.rollup_seq_url.clone(),
    ))
    .await;
    log::debug!("ERC20 Legacy Class Hash Declared !!!");
    save_to_json("erc20_legacy_class_hash", &JsonValueType::StringType(erc20_legacy_class_hash.to_string())).unwrap();
    sleep(Duration::from_secs(10)).await;

    let legacy_eth_bridge_class_hash = declare_contract_util_func(DeclarationInput::LegacyDeclarationInputs(
        String::from(LEGACY_BRIDGE_PATH),
        arg_config.rollup_seq_url.clone(),
    ))
    .await;
    log::debug!("Legacy ETH Bridge Class Hash Declared !!!");
    save_to_json("legacy_eth_bridge_class_hash", &JsonValueType::StringType(legacy_eth_bridge_class_hash.to_string()))
        .unwrap();
    sleep(Duration::from_secs(10)).await;

    let eth_proxy_address = deploy_proxy_contract(
        &user_account,
        account_address,
        legacy_proxy_class_hash,
        // salt taken from : https://sepolia.starkscan.co/tx/0x06a5a493cf33919e58aa4c75777bffdef97c0e39cac968896d7bee8cc67905a1
        FieldElement::from_str("0x322c2610264639f6b2cee681ac53fa65c37e187ea24292d1b21d859c55e1a78").unwrap(),
        FieldElement::ONE,
    )
    .await;
    log::info!("ETH Proxy Address : {:?}", eth_proxy_address);
    save_to_json("l2_eth_address_proxy", &JsonValueType::StringType(eth_proxy_address.to_string())).unwrap();
    sleep(Duration::from_secs(10)).await;

    let eth_bridge_proxy_address = deploy_proxy_contract(
        &user_account,
        account_address,
        legacy_proxy_class_hash,
        FieldElement::from_str("0xabcdabcdabcd").unwrap(),
        FieldElement::ZERO,
    )
    .await;
    log::info!("ETH Bridge Proxy Address : {:?}", eth_bridge_proxy_address);
    save_to_json("ETH_l2_bridge_address_proxy", &JsonValueType::StringType(eth_bridge_proxy_address.to_string()))
        .unwrap();
    sleep(Duration::from_secs(10)).await;

    init_governance_proxy(&user_account, eth_proxy_address, "eth_proxy_address : init_governance_proxy").await;
    sleep(Duration::from_secs(10)).await;
    init_governance_proxy(&user_account, eth_bridge_proxy_address, "eth_bridge_proxy_address : init_governance_proxy")
        .await;
    sleep(Duration::from_secs(10)).await;

    EthBridgeInitOutput {
        legacy_proxy_class_hash,
        starkgate_proxy_class_hash,
        erc20_legacy_class_hash,
        legacy_eth_bridge_class_hash,
        eth_proxy_address,
        eth_bridge_proxy_address,
    }
}
