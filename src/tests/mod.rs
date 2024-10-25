pub mod constants;
mod erc20_bridge;
mod eth_bridge;
mod madara;

use constants::{
    APP_CHAIN_ID, ETH_CHAIN_ID, ETH_PRIV_KEY, ETH_RPC, FEE_TOKEN_ADDRESS, L1_DEPLOYER_ADDRESS, L1_WAIT_TIME,
    NATIVE_FEE_TOKEN_ADDRESS, ROLLUP_PRIV_KEY, ROLLUP_SEQ_URL, SN_OS_CONFIG_HASH_VERSION, SN_OS_PROGRAM_HASH,
};
use rstest::rstest;

use crate::contract_clients::config::Config;
use crate::tests::constants::{L1_MULTISIG_ADDRESS, L2_MULTISIG_ADDRESS, OPERATOR_ADDRESS, VERIFIER_ADDRESS};
use crate::tests::erc20_bridge::erc20_bridge_test_helper;
use crate::tests::eth_bridge::eth_bridge_test_helper;
use crate::tests::madara::{MadaraCmd, MadaraCmdBuilder};
use crate::{
    bootstrap, get_account, setup_argent, setup_braavos, setup_core_contract, setup_erc20_bridge, setup_eth_bridge,
    setup_udc, BootstrapperOutput, CliArgs,
};

async fn test_setup(config: &CliArgs, clients: &Config) -> (BootstrapperOutput, MadaraCmd) {
    // setup the core contract first, given it would start on a empty anvil, the address would be same
    let core_contract_client = setup_core_contract(config, clients).await;

    // run the madara with the commans:
    // cargo run --release -- --name madara --base-path ../madara_db --rpc-port 9944 --rpc-cors "*"
    // --rpc-external --sequencer --chain-config-path configs/presets/devnet.yaml --gas-price 0
    // --blob-gas-price 0 --strk-gas-price 0 --strk-blob-gas-price 0 --rpc-methods unsafe
    // --feeder-gateway-enable --gateway-enable --gateway-external --no-l1-sync
    let mut node = MadaraCmdBuilder::new()
        .args([
            "--no-sync-polling",
            "--l1-endpoint",
            "http://localhost:8545",
            "--chain-config-path=./src/tests/devnet.yml",
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

    // setup the rest of the stuff and return the ideal output for the tests

    // setup the account
    let account = get_account(clients, config).await;

    // setup eth bridge
    let eth_bridge_setup_outputs =
        setup_eth_bridge(Some(account.clone()), &core_contract_client, config, clients).await;

    // setup erc20 bridge
    let erc20_bridge_setup_outputs =
        setup_erc20_bridge(Some(account.clone()), &core_contract_client, config, clients).await;

    // setup udc
    let udc_setup_outputs = setup_udc(Some(account.clone()), config, clients).await;

    // setup argent account
    let argent_setup_outputs = setup_argent(Some(account.clone()), config, clients).await;

    // setup braavos account
    let braavos_setup_outputs = setup_braavos(Some(account.clone()), config, clients).await;

    (
        BootstrapperOutput {
            starknet_contract_address: Some(core_contract_client.core_contract_client.address()),
            starknet_contract_implementation_address: Some(
                core_contract_client.core_contract_client.implementation_address(),
            ),
            eth_bridge_setup_outputs: Some(eth_bridge_setup_outputs),
            erc20_bridge_setup_outputs: Some(erc20_bridge_setup_outputs),
            udc_setup_outputs: Some(udc_setup_outputs),
            argent_setup_outputs: Some(argent_setup_outputs),
            braavos_setup_outputs: Some(braavos_setup_outputs),
        },
        node,
    )
}

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
    let eth_bridge_setup = out.eth_bridge_setup_outputs.unwrap();

    let _ = eth_bridge_test_helper(
        &clients,
        &get_config(),
        eth_bridge_setup.l2_eth_proxy_address,
        eth_bridge_setup.l2_eth_bridge_proxy_address,
        eth_bridge_setup.l1_bridge_address,
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
    let eth_token_setup = out.erc20_bridge_setup_outputs.unwrap();

    let _ = erc20_bridge_test_helper(
        &clients,
        &get_config(),
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
    let clients = Config::init(&get_config()).await;
    let (out, _madara) = test_setup(&get_config(), &clients).await;

    let eth_bridge_setup = out.eth_bridge_setup_outputs.unwrap();
    let eth_token_setup = out.erc20_bridge_setup_outputs.unwrap();

    let _ = eth_bridge_test_helper(
        &clients,
        &get_config(),
        eth_bridge_setup.l2_eth_proxy_address,
        eth_bridge_setup.l2_eth_bridge_proxy_address,
        eth_bridge_setup.l1_bridge_address,
    )
    .await;

    let _ = erc20_bridge_test_helper(
        &clients,
        &get_config(),
        eth_token_setup.test_erc20_token_address,
        eth_token_setup.token_bridge,
        eth_token_setup.l2_token_bridge,
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
