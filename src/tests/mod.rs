pub mod constants;
mod erc20_bridge;
mod eth_bridge;

use constants::{
    APP_CHAIN_ID, ETH_CHAIN_ID, ETH_PRIV_KEY, ETH_RPC, FEE_TOKEN_ADDRESS, L1_DEPLOYER_ADDRESS, L1_WAIT_TIME,
    NATIVE_FEE_TOKEN_ADDRESS, ROLLUP_PRIV_KEY, ROLLUP_SEQ_URL, SN_OS_CONFIG_HASH_VERSION, SN_OS_PROGRAM_HASH,
};
use rstest::rstest;

use crate::contract_clients::config::Config;
use crate::tests::constants::{L1_MULTISIG_ADDRESS, L2_MULTISIG_ADDRESS, OPERATOR_ADDRESS, VERIFIER_ADDRESS};
use crate::tests::erc20_bridge::erc20_bridge_test_helper;
use crate::tests::eth_bridge::eth_bridge_test_helper;
use crate::{bootstrap, CliArgs};

#[rstest]
#[tokio::test]
#[ignore]
async fn deploy_bridge() -> Result<(), anyhow::Error> {
    env_logger::init();
    let clients = Config::init(&get_config()).await;
    bootstrap(&get_config(), &clients).await;

    Ok(())
}

#[rstest]
#[tokio::test]
#[ignore]
async fn deposit_and_withdraw_eth_bridge() -> Result<(), anyhow::Error> {
    env_logger::init();
    let clients = Config::init(&get_config()).await;
    let out = bootstrap(&get_config(), &clients).await;

    let _ = eth_bridge_test_helper(
        &clients,
        &get_config(),
        out.eth_bridge_setup_outputs
            .ok_or_else(|| anyhow::anyhow!("eth_bridge_setup_outputs is None"))?
            .l2_eth_proxy_address,
        out.eth_bridge_setup_outputs
            .ok_or_else(|| anyhow::anyhow!("eth_bridge_setup_outputs is None"))?
            .l2_eth_bridge_proxy_address,
        out.eth_bridge_setup_outputs
            .ok_or_else(|| anyhow::anyhow!("eth_bridge_setup_outputs is None"))?
            .l1_bridge,
    )
    .await;

    Ok(())
}

#[rstest]
#[tokio::test]
#[ignore]
async fn deposit_and_withdraw_erc20_bridge() -> Result<(), anyhow::Error> {
    env_logger::init();
    let clients = Config::init(&get_config()).await;
    let out = bootstrap(&get_config(), &clients).await;

    let erc20_bridge_setup = out.erc20_bridge_setup_outputs.as_ref()
    .ok_or_else(|| anyhow::anyhow!("erc20_bridge_setup_outputs is None"))?;
    
    let _ = erc20_bridge_test_helper(
        &clients,
        &get_config(),
        erc20_bridge_setup.test_erc20_token_address,
        erc20_bridge_setup.token_bridge,
        erc20_bridge_setup.l2_token_bridge,
    )
    .await;

    Ok(())
}

#[rstest]
#[tokio::test]
async fn deposit_tests_both_bridges() -> Result<(), anyhow::Error> {
    env_logger::init();
    let clients = Config::init(&get_config()).await;
    let out = bootstrap(&get_config(), &clients).await;

    let _ = eth_bridge_test_helper(
        &clients,
        &get_config(),
        out.eth_bridge_setup_outputs
            .ok_or_else(|| anyhow::anyhow!("eth_bridge_setup_outputs is None"))?
            .l2_eth_proxy_address,
        out.eth_bridge_setup_outputs
            .ok_or_else(|| anyhow::anyhow!("eth_bridge_setup_outputs is None"))?
            .l2_eth_bridge_proxy_address,
        out.eth_bridge_setup_outputs
            .ok_or_else(|| anyhow::anyhow!("eth_bridge_setup_outputs is None"))?
            .l1_bridge,
    )
    .await;

    let _ = erc20_bridge_test_helper(
        &clients,
        &get_config(),
        out.erc20_bridge_setup_outputs
            .ok_or_else(|| anyhow::anyhow!("erc20_bridge_setup_outputs is None"))?
            .test_erc20_token_address,
        out.erc20_bridge_setup_outputs
            .ok_or_else(|| anyhow::anyhow!("erc20_bridge_setup_outputs is None"))?
            .token_bridge,
        out.erc20_bridge_setup_outputs
            .ok_or_else(|| anyhow::anyhow!("erc20_bridge_setup_outputs is None"))?
            .l2_token_bridge,
    )
    .await;

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
        mode: crate::BootstrapMode::Full,
        core_contract_address: None,
        core_contract_implementation_address: None,
        output_file: None,
    }
}
