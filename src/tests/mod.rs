pub mod constants;
mod erc20_bridge;
mod eth_bridge;

use std::process::Command;
use std::time::Duration;
use std::{env, fs};

use rstest::rstest;
use tokio::time::sleep;

use crate::contract_clients::config::Clients;
use crate::tests::erc20_bridge::erc20_bridge_test_helper;
use crate::tests::eth_bridge::eth_bridge_test_helper;
use crate::{bootstrap, setup_core_contract, setup_l2, BootstrapperOutput, ConfigFile};

async fn test_setup(args: &ConfigFile, clients: &Clients) -> BootstrapperOutput {
    // Setup L1 (core contract)
    let core_contract_client = setup_core_contract(args, clients).await;

    let core_contract_address = core_contract_client.core_contract_client.address();
    let core_contract_implementation_address = core_contract_client.core_contract_client.implementation_address();

    // Create a new config with the core contract addresses
    let mut config = get_test_config_file();
    config.core_contract_address = Some(format!("{:?}", core_contract_address));
    config.core_contract_implementation_address = Some(format!("{:?}", core_contract_implementation_address));

    ensure_toolchain().expect("Not able to ensure toolchain exists.");
    wait_for_madara().await.expect("Failed to start madara!");

    // Setup L2 with the updated config
    let l2_output = setup_l2(&config, clients).await;

    BootstrapperOutput {
        starknet_contract_address: Some(core_contract_address),
        starknet_contract_implementation_address: Some(core_contract_implementation_address),
        ..l2_output
    }
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

    // This will kill the madara when this test fails/passes
    let _port_killer = PortKiller;

    let clients = Clients::init_from_config(&config).await;
    let out = test_setup(&config, &clients).await;

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

// Create a struct that will kill the process on port when dropped
struct PortKiller;
impl Drop for PortKiller {
    fn drop(&mut self) {
        kill_process_on_port(19944);
    }
}

fn kill_process_on_port(port: u16) {
    Command::new("sh")
        .arg("-c")
        .arg(format!("lsof -i :{} | grep LISTEN | awk '{{print $2}}' | xargs kill -9", port))
        .output()
        .expect("Failed to execute command");
}

fn get_test_config_file() -> ConfigFile {
    ConfigFile::default()
}

async fn wait_for_madara() -> color_eyre::Result<()> {
    env::set_current_dir("madara").expect("madara folder doesn't exist.");
    fs::create_dir_all("../madara-dbs").expect("unable to create folders");
    env::set_var("RUST_LOG", "info");

    Command::new("cargo")
        .arg("+1.81")
        .arg("run")
        .arg("--release")
        .arg("--")
        .arg("--name")
        .arg("madara")
        .arg("--base-path")
        .arg("../madara-dbs/madara_pathfinder_test_11")
        .arg("--rpc-port")
        .arg("19944")
        .arg("--rpc-cors")
        .arg("*")
        .arg("--rpc-external")
        .arg("--sequencer")
        .arg("--chain-config-path")
        .arg("configs/presets/devnet.yaml")
        .arg("--feeder-gateway-enable")
        .arg("--gateway-enable")
        .arg("--gateway-external")
        .arg("--gas-price")
        .arg("0")
        .arg("--blob-gas-price")
        .arg("0")
        .arg("--rpc-methods")
        .arg("unsafe")
        .arg("--l1-endpoint")
        .arg("http://localhost:8545")
        .spawn()?;

    env::set_current_dir("../").expect("Navigate back failed.");

    // Madara build time (approx : 20 mins.)
    sleep(Duration::from_secs(1200)).await;

    Ok(())
}

fn ensure_toolchain() -> color_eyre::Result<()> {
    let output = Command::new("rustup").arg("toolchain").arg("list").output()?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    if !output_str.contains("1.81") {
        Command::new("rustup").arg("install").arg("1.81").status()?;
    }
    Ok(())
}
