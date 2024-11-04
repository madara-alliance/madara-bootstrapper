pub mod constants;
mod erc20_bridge;
mod eth_bridge;
mod madara;

use constants::{
    APP_CHAIN_ID, ETH_CHAIN_ID, ETH_PRIV_KEY, ETH_RPC, FEE_TOKEN_ADDRESS, L1_DEPLOYER_ADDRESS, L1_WAIT_TIME,
    NATIVE_FEE_TOKEN_ADDRESS, ROLLUP_PRIV_KEY, ROLLUP_SEQ_URL, SN_OS_CONFIG_HASH_VERSION, SN_OS_PROGRAM_HASH,
};
use rstest::rstest;

use crate::contract_clients::config::Clients;
use crate::tests::constants::{L1_MULTISIG_ADDRESS, L2_MULTISIG_ADDRESS, OPERATOR_ADDRESS, VERIFIER_ADDRESS};
use crate::tests::erc20_bridge::erc20_bridge_test_helper;
use crate::tests::eth_bridge::eth_bridge_test_helper;
use crate::tests::madara::{MadaraCmd, MadaraCmdBuilder};
use crate::{bootstrap, setup_core_contract, setup_l2, BootstrapperOutput, ConfigFile};

async fn test_setup(args: &ConfigFile, clients: &Clients) -> (BootstrapperOutput, MadaraCmd) {
    // Setup L1 (core contract)
    let core_contract_client = setup_core_contract(args, clients).await;

    let core_contract_address = core_contract_client.core_contract_client.address();
    let core_contract_implementation_address = core_contract_client.core_contract_client.implementation_address();

    // Create a new config with the core contract addresses
    let mut config = get_test_config_file();
    config.core_contract_address = Some(format!("{:?}", core_contract_address));
    config.core_contract_implementation_address = Some(format!("{:?}", core_contract_implementation_address));

    let mut node = MadaraCmdBuilder::new()
        .args([
            "--no-sync-polling",
            "--l1-endpoint",
            "http://localhost:8545",
            "--chain-config-path=./bin/devnet.yaml",
            "--rpc-cors",
            "*",
            "--rpc-external",
            "--sequencer",
            "--feeder-gateway-enable",
            "--gateway-enable",
            "--gateway-external",
            "--rpc-methods",
            "unsafe",
            "--gas-price",
            "0",
            "--blob-gas-price",
            "0",
            "--strk-gas-price",
            "0",
            "--strk-blob-gas-price",
            "0",
        ])
        .run();
    node.wait_for_ready().await;

    // Setup L2 with the updated config
    let l2_output = setup_l2(&config, clients).await;

    let output = BootstrapperOutput {
        starknet_contract_address: Some(core_contract_address),
        starknet_contract_implementation_address: Some(core_contract_implementation_address),
        ..l2_output
    };

    (output, node)
}

#[rstest]
#[tokio::test]
#[ignore = "ignored because we have a e2e test, and this is for a local test"]
async fn deploy_bridge() -> Result<(), anyhow::Error> {
    env_logger::init();
    let config = get_test_config_file();
    let clients = Clients::init_from_config(&config).await;
    bootstrap(&config, &clients).await;

    Ok(())
}

#[rstest]
#[tokio::test]
#[ignore = "ignored because we have a e2e test, and this is for a local test"]
async fn deposit_and_withdraw_eth_bridge() -> Result<(), anyhow::Error> {
    env_logger::init();
    let config = get_test_config_file();
    let clients = Clients::init_from_config(&config).await;
    let out = bootstrap(&config, &clients).await;
    let eth_bridge_setup = out.eth_bridge_setup_outputs.unwrap();

    let _ = eth_bridge_test_helper(
        &clients,
        &config,
        eth_bridge_setup.l2_eth_proxy_address,
        eth_bridge_setup.l2_eth_bridge_proxy_address,
        eth_bridge_setup.l1_bridge,
    )
    .await;

    Ok(())
}

#[rstest]
#[tokio::test]
#[ignore = "ignored because we have a e2e test, and this is for a local test"]
async fn deposit_and_withdraw_erc20_bridge() -> Result<(), anyhow::Error> {
    env_logger::init();
    let config = get_test_config_file();
    let clients = Clients::init_from_config(&config).await;
    let out = bootstrap(&config, &clients).await;
    let eth_token_setup = out.erc20_bridge_setup_outputs.unwrap();

    let _ = erc20_bridge_test_helper(
        &clients,
        &config,
        eth_token_setup.test_erc20_token_address,
        eth_token_setup.token_bridge,
        eth_token_setup.l2_token_bridge,
    )
    .await;

    Ok(())
}

#[rstest]
#[tokio::test]
async fn deposit_tests_both_bridges() -> Result<(), anyhow::Error> {
    env_logger::init();
    let config = get_test_config_file();
    let clients = Clients::init_from_config(&config).await;
    let (out, _madara) = test_setup(&config, &clients).await;

    let eth_bridge_setup = out.eth_bridge_setup_outputs.unwrap();
    let eth_token_setup = out.erc20_bridge_setup_outputs.unwrap();

    let _ = eth_bridge_test_helper(
        &clients,
        &config,
        eth_bridge_setup.l2_eth_proxy_address,
        eth_bridge_setup.l2_eth_bridge_proxy_address,
        eth_bridge_setup.l1_bridge,
    )
    .await;

    let _ = erc20_bridge_test_helper(
        &clients,
        &config,
        eth_token_setup.test_erc20_token_address,
        eth_token_setup.token_bridge,
        eth_token_setup.l2_token_bridge,
    )
    .await;

    Ok(())
}

fn get_test_config_file() -> ConfigFile {
    ConfigFile {
        eth_rpc: String::from(ETH_RPC),
        eth_priv_key: String::from(ETH_PRIV_KEY),
        rollup_seq_url: String::from(ROLLUP_SEQ_URL),
        rollup_priv_key: String::from(ROLLUP_PRIV_KEY),
        eth_chain_id: ETH_CHAIN_ID.parse().expect("Invalid ETH chain ID"),
        l1_deployer_address: String::from(L1_DEPLOYER_ADDRESS),
        l1_wait_time: String::from(L1_WAIT_TIME),
        sn_os_program_hash: String::from(SN_OS_PROGRAM_HASH),
        config_hash_version: String::from(SN_OS_CONFIG_HASH_VERSION),
        app_chain_id: String::from(APP_CHAIN_ID),
        fee_token_address: String::from(FEE_TOKEN_ADDRESS),
        native_fee_token_address: String::from(NATIVE_FEE_TOKEN_ADDRESS),
        cross_chain_wait_time: 20,
        l1_multisig_address: String::from(L1_MULTISIG_ADDRESS),
        l2_multisig_address: String::from(L2_MULTISIG_ADDRESS),
        verifier_address: String::from(VERIFIER_ADDRESS),
        operator_address: String::from(OPERATOR_ADDRESS),
        dev: false,
        core_contract_address: None,
        core_contract_implementation_address: None,
    }
}
