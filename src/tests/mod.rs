mod helpers;

use crate::bridge::contract_clients::config::{
    generate_config_hash, get_bridge_init_configs, Config,
};
use crate::bridge::contract_clients::starknet_sovereign::StarknetSovereignContract;
use crate::deploy_bridges;
use crate::tests::helpers::erc20_bridge_test_helper;
use crate::utils::arg_config::ArgConfig;
use clap::Parser;
use ethers::abi::AbiEncode;
use helpers::eth_bridge_test_helper;
use hex::encode;
use hex::ToHex;
use rstest::rstest;
use starknet_ff::FieldElement;
use std::io::Write;
use std::process;

#[rstest]
#[tokio::test]
#[ignore]
async fn deploy_bridge() -> Result<(), anyhow::Error> {
    env_logger::init();

    let config = ArgConfig::test().unwrap_or_else(|err| {
        log::error!("Problem parsing args ❌ : {}", err);
        process::exit(1)
    });

    deploy_bridges(&config).await;

    Ok(())
}

#[rstest]
#[tokio::test]
// #[ignore]
async fn deposit_and_withdraw_eth_bridge() -> Result<(), anyhow::Error> {
    env_logger::init();

    let config = ArgConfig::test().unwrap_or_else(|err| {
        log::error!("Problem parsing args ❌ : {}", err);
        process::exit(1)
    });

    log::debug!("{:?}", &config.config_hash_version);

    let clients = Config::init(&config).await;
    let core_contract_client = StarknetSovereignContract::deploy(&clients).await;
    log::debug!("core address [📦] : {:?}", core_contract_client.address());
    let (program_hash, config_hash) = get_bridge_init_configs(&config);
    core_contract_client
        .initialize_for_goerli(0u64.into(), 0u64.into(), program_hash, config_hash)
        .await;
    log::debug!("bridge init for goerli successful [✅]");

    let _ = eth_bridge_test_helper(&clients, &config, &core_contract_client).await;

    Ok(())
}

#[rstest]
#[tokio::test]
#[ignore]
async fn deposit_and_withdraw_erc20_bridge() -> Result<(), anyhow::Error> {
    env_logger::init();

    let config = ArgConfig::test().unwrap_or_else(|err| {
        log::error!("Problem parsing args ❌ : {}", err);
        process::exit(1)
    });

    let clients = Config::init(&config).await;
    let core_contract_client = StarknetSovereignContract::deploy(&clients).await;
    log::debug!("core address [📦] : {:?}", core_contract_client.address());
    let (program_hash, config_hash) = get_bridge_init_configs(&config);
    core_contract_client
        .initialize_for_goerli(0u64.into(), 0u64.into(), program_hash, config_hash)
        .await;
    log::debug!("bridge init for goerli successful [✅]");
    let _ = erc20_bridge_test_helper(&clients, &config, &core_contract_client).await;

    Ok(())
}
