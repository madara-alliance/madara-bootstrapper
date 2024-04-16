pub mod constants;
mod erc20_bridge;
mod eth_bridge;

use constants::{
    APP_CHAIN_ID, ETH_CHAIN_ID, ETH_PRIV_KEY, ETH_RPC, FEE_TOKEN_ADDRESS, L1_DEPLOYER_ADDRESS, L1_WAIT_TIME,
    L2_DEPLOYER_ADDRESS, ROLLUP_PRIV_KEY, ROLLUP_SEQ_URL, SN_OS_CONFIG_HASH_VERSION, SN_OS_PROGRAM_HASH,
};
use rstest::rstest;

use crate::contract_clients::config::Config;
use crate::contract_clients::starknet_sovereign::StarknetSovereignContract;
use crate::contract_clients::utils::get_bridge_init_configs;
use crate::tests::erc20_bridge::erc20_bridge_test_helper;
use crate::tests::eth_bridge::eth_bridge_test_helper;
use crate::{deploy_bridges, CliArgs};

#[rstest]
#[tokio::test]
#[ignore]
async fn deploy_bridge() -> Result<(), anyhow::Error> {
    env_logger::init();

    deploy_bridges(&get_config()).await;

    Ok(())
}

#[rstest]
#[tokio::test]
#[ignore]
async fn deposit_and_withdraw_eth_bridge() -> Result<(), anyhow::Error> {
    env_logger::init();

    let clients = Config::init(&get_config()).await;
    let core_contract_client = StarknetSovereignContract::deploy(&clients).await;
    log::debug!("Core address [📦] : {:?}", core_contract_client.address());
    let (program_hash, config_hash) = get_bridge_init_configs(&get_config());
    core_contract_client.initialize_core_contract(0u64.into(), 0u64.into(), program_hash, config_hash).await;
    log::debug!("Bridge init successful [✅]");

    let _ = eth_bridge_test_helper(&clients, &get_config(), &core_contract_client).await;

    Ok(())
}

#[rstest]
#[tokio::test]
// #[ignore]
async fn deposit_and_withdraw_erc20_bridge() -> Result<(), anyhow::Error> {
    env_logger::init();

    let clients = Config::init(&get_config()).await;
    let core_contract_client = StarknetSovereignContract::deploy(&clients).await;
    log::debug!("Core address [📦] : {:?}", core_contract_client.address());
    let (program_hash, config_hash) = get_bridge_init_configs(&get_config());
    core_contract_client.initialize_core_contract(0u64.into(), 0u64.into(), program_hash, config_hash).await;
    log::debug!("Bridge init successful [✅]");
    let _ = erc20_bridge_test_helper(&clients, &get_config(), &core_contract_client).await;

    Ok(())
}

fn get_config() -> CliArgs {
    CliArgs {
        eth_rpc: String::from(ETH_RPC),
        eth_priv_key: String::from(ETH_PRIV_KEY),
        rollup_seq_url: String::from(ROLLUP_SEQ_URL),
        rollup_priv_key: String::from(ROLLUP_PRIV_KEY),
        eth_chain_id: String::from(ETH_CHAIN_ID).parse().unwrap(),
        l1_deployer_address: String::from(L1_DEPLOYER_ADDRESS),
        l2_deployer_address: String::from(L2_DEPLOYER_ADDRESS),
        l1_wait_time: String::from(L1_WAIT_TIME),
        sn_os_program_hash: String::from(SN_OS_PROGRAM_HASH),
        config_hash_version: String::from(SN_OS_CONFIG_HASH_VERSION),
        app_chain_id: String::from(APP_CHAIN_ID),
        fee_token_address: String::from(FEE_TOKEN_ADDRESS),
        cross_chain_wait_time: 120,
    }
}
