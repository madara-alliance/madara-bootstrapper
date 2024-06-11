pub mod bridge;
pub mod contract_clients;
pub mod non_bridge;
mod setup_scripts;
#[cfg(test)]
pub mod tests;
pub mod utils;

use clap::Parser;
use dotenv::dotenv;
use starknet_accounts::Account;
use starknet_ff::FieldElement;

use crate::bridge::deploy_erc20_bridge::deploy_erc20_bridge;
use crate::bridge::deploy_eth_bridge::deploy_eth_bridge;
use crate::contract_clients::config::Config;
use crate::contract_clients::starknet_sovereign::StarknetSovereignContract;
use crate::contract_clients::token_bridge::StarknetTokenBridge;
use crate::contract_clients::utils::get_bridge_init_configs;
use crate::non_bridge::deployer::deploy_non_bridge_contracts;
use crate::setup_scripts::account_setup::account_init_func;
use crate::setup_scripts::erc20_bridge::erc20_bridge_init_func;
use crate::setup_scripts::eth_bridge::eth_bridge_init_func;
use crate::utils::{convert_to_hex, save_to_json, JsonValueType};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    #[clap(long, env, default_value = "http://127.0.0.1:8545")]
    eth_rpc: String,
    #[clap(long, env, default_value = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80")]
    eth_priv_key: String,
    #[clap(long, env, default_value = "https://bf8f-2405-201-4059-e00f-3986-3c9f-2139-bf11.ngrok-free.app")]
    rollup_seq_url: String,
    #[clap(long, env, default_value = "0xabcd")]
    rollup_priv_key: String,
    #[clap(long, env, default_value_t = 31337)]
    eth_chain_id: u64,
    #[clap(long, env, default_value = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266")]
    l1_deployer_address: String,
    #[clap(long, env, default_value = "15")]
    l1_wait_time: String,
    #[clap(long, env, default_value = "0x41fc2a467ef8649580631912517edcab7674173f1dbfa2e9b64fbcd82bc4d79")]
    sn_os_program_hash: String,
    #[clap(long, env, default_value = "StarknetOsConfig1")]
    config_hash_version: String,
    #[clap(long, env, default_value = "MADARA")]
    app_chain_id: String,
    #[clap(long, env, default_value = "0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7")]
    fee_token_address: String,
    #[clap(long, env, default_value_t = 80)]
    cross_chain_wait_time: u64,
}

#[tokio::main]
pub async fn main() {
    env_logger::init();
    dotenv().ok();

    let args = CliArgs::parse();

    deploy_bridges(&args).await;
}

pub struct DeployBridgeOutput {
    pub starknet_sovereign_contract: StarknetSovereignContract,
    pub starknet_token_bridge: StarknetTokenBridge,
    pub erc20_class_hash: FieldElement,
    pub legacy_eth_bridge_class_hash: FieldElement,
    pub account_address: FieldElement,
    pub eth_proxy_address: FieldElement,
    pub eth_bridge_proxy_address: FieldElement,
    pub legacy_proxy_class_hash: FieldElement,
    pub starkgate_proxy_class_hash: FieldElement,
    pub erc20_legacy_class_hash: FieldElement,
    pub erc20_l2_bridge_address: FieldElement,
    pub l2_erc20_token_address: FieldElement,
}

pub async fn deploy_bridges(config: &CliArgs) -> DeployBridgeOutput {
    let clients = Config::init(config).await;
    let core_contract_client = StarknetSovereignContract::deploy(&clients).await;
    log::info!("Core address [📦] : {:?}", core_contract_client.address());
    save_to_json("l1_core_contract_address", &JsonValueType::EthAddress(core_contract_client.address())).unwrap();
    let (program_hash, config_hash) = get_bridge_init_configs(config);
    core_contract_client.initialize_core_contract(0u64.into(), 0u64.into(), program_hash, config_hash).await;
    log::info!("Bridge init for l1 successful [✅]");
    log::debug!(">>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> [L2 State and Initialisation] ⏳");
    let account = account_init_func(&clients, config).await;
    let eth_bridge_setup_outputs = eth_bridge_init_func(config, account.clone(), account.address()).await;
    let erc20_bridge_setup_outputs = erc20_bridge_init_func(account).await;
    log::debug!(">>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> [ETH BRIDGE] ⏳");
    deploy_eth_bridge(
        &clients,
        config,
        &core_contract_client,
        eth_bridge_setup_outputs.legacy_eth_bridge_class_hash,
        eth_bridge_setup_outputs.eth_bridge_proxy_address,
        eth_bridge_setup_outputs.eth_proxy_address,
        account.address().clone(),
        eth_bridge_setup_outputs.starkgate_proxy_class_hash,
        eth_bridge_setup_outputs.erc20_legacy_class_hash,
    )
    .await
    .expect("Error in deploying ETH bridge");
    log::debug!("ETH BRIDGE DEPLOYED [✅]");
    log::debug!(">>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>[ERC20 BRIDGE] ⏳");
    let (starknet_token_bridge, erc20_l2_bridge_address, l2_erc20_token_address) = deploy_erc20_bridge(
        &clients,
        config,
        &core_contract_client,
        &convert_to_hex(&account.address().clone().to_string()),
        erc20_bridge_setup_outputs.erc20_cairo_one_class_hash,
    )
    .await
    .expect(
        "Error in
    deploying ERC20 bridge",
    );
    log::debug!("ERC20 BRIDGE DEPLOYED [✅]");
    log::debug!(">>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>[NON BRIDGE CONTRACTS] ⏳");
    deploy_non_bridge_contracts(&clients, config, account.address().clone()).await;
    log::debug!("NON BRIDGE CONTRACTS DEPLOYED [✅]");

    DeployBridgeOutput {
        starknet_sovereign_contract: core_contract_client,
        starknet_token_bridge,
        erc20_class_hash: erc20_bridge_setup_outputs.erc20_cairo_one_class_hash,
        legacy_eth_bridge_class_hash: eth_bridge_setup_outputs.legacy_eth_bridge_class_hash,
        account_address: account.address().clone(),
        eth_proxy_address: eth_bridge_setup_outputs.eth_proxy_address,
        eth_bridge_proxy_address: eth_bridge_setup_outputs.eth_bridge_proxy_address,
        legacy_proxy_class_hash: eth_bridge_setup_outputs.legacy_proxy_class_hash,
        starkgate_proxy_class_hash: eth_bridge_setup_outputs.starkgate_proxy_class_hash,
        erc20_legacy_class_hash: eth_bridge_setup_outputs.erc20_legacy_class_hash,
        erc20_l2_bridge_address,
        l2_erc20_token_address,
    }
}
