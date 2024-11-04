pub mod contract_clients;
pub mod helpers;
mod setup_scripts;
#[cfg(test)]
pub mod tests;
pub mod utils;

use std::fs::File;
use std::path::PathBuf;
use std::str::FromStr;

use clap::{Parser, ValueEnum};
use contract_clients::utils::RpcAccount;
use dotenv::dotenv;
use ethers::abi::Address;
use inline_colorization::*;
use serde::{Deserialize, Serialize};
use setup_scripts::argent::ArgentSetupOutput;
use setup_scripts::braavos::BraavosSetupOutput;
use setup_scripts::core_contract::CoreContractStarknetL1Output;
use setup_scripts::erc20_bridge::Erc20BridgeSetupOutput;
use setup_scripts::eth_bridge::EthBridgeSetupOutput;
use setup_scripts::udc::UdcSetupOutput;
use starknet::accounts::Account;
use starknet_core_contract_client::clients::StarknetValidityContractClient;

use crate::contract_clients::config::Clients;
use crate::contract_clients::starknet_validity::StarknetValidityContract;
use crate::setup_scripts::account_setup::account_init;
use crate::setup_scripts::argent::ArgentSetup;
use crate::setup_scripts::braavos::BraavosSetup;
use crate::setup_scripts::core_contract::CoreContractStarknetL1;
use crate::setup_scripts::erc20_bridge::Erc20Bridge;
use crate::setup_scripts::eth_bridge::EthBridge;
use crate::setup_scripts::udc::UdcSetup;
use crate::utils::banner::BANNER;
use crate::utils::{save_to_json, JsonValueType};

#[derive(Debug, Clone, Copy, ValueEnum)]
enum BootstrapMode {
    Core,
    SetupL1,
    SetupL2,
    EthBridge,
    Erc20Bridge,
    Udc,
    Argent,
    Braavos,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    #[clap(long)]
    config: Option<PathBuf>,
    #[clap(long, env, value_enum)]
    mode: BootstrapMode,
    #[clap(long, env)]
    output_file: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ConfigFile {
    pub eth_rpc: String,
    pub eth_priv_key: String,
    pub rollup_seq_url: String,
    pub rollup_priv_key: String,
    pub eth_chain_id: u64,
    pub l1_deployer_address: String,
    pub l1_wait_time: String,
    pub sn_os_program_hash: String,
    pub config_hash_version: String,
    pub app_chain_id: String,
    pub fee_token_address: String,
    pub native_fee_token_address: String,
    pub cross_chain_wait_time: u64,
    pub l1_multisig_address: String,
    pub l2_multisig_address: String,
    pub verifier_address: String,
    pub operator_address: String,
    pub dev: bool,
    pub core_contract_address: Option<String>,
    pub core_contract_implementation_address: Option<String>,
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            eth_rpc: "http://127.0.0.1:8545".to_string(),
            eth_priv_key: "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".to_string(),
            rollup_seq_url: "http://127.0.0.1:9944".to_string(),
            rollup_priv_key: "0xabcd".to_string(),
            eth_chain_id: 31337,
            l1_deployer_address: "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266".to_string(),
            l1_wait_time: "15".to_string(),
            sn_os_program_hash: "0x41fc2a467ef8649580631912517edcab7674173f1dbfa2e9b64fbcd82bc4d79".to_string(),
            config_hash_version: "StarknetOsConfig2".to_string(),
            app_chain_id: "MADARA_DEVNET".to_string(),
            fee_token_address: "0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7".to_string(),
            native_fee_token_address: "0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d".to_string(),
            cross_chain_wait_time: 20,
            l1_multisig_address: "0x70997970C51812dc3A010C7d01b50e0d17dc79C8".to_string(),
            l2_multisig_address: "0x556455b8ac8bc00e0ad061d7df5458fa3c372304877663fa21d492a8d5e9435".to_string(),
            verifier_address: "0x000000000000000000000000000000000000abcd".to_string(),
            operator_address: "0x000000000000000000000000000000000000abcd".to_string(),
            dev: false,
            core_contract_address: None,
            core_contract_implementation_address: None,
        }
    }
}

#[tokio::main]
pub async fn main() {
    env_logger::init();
    dotenv().ok();

    let args = CliArgs::parse();

    println!("{color_red}{}{color_reset}", BANNER);

    // Load config from file or use defaults
    let config_file = match args.config {
        Some(path) => {
            let file = File::open(path).expect("Failed to open config file");
            serde_json::from_reader(file).expect("Failed to parse config file")
        }
        None => ConfigFile::default(),
    };

    let clients = Clients::init_from_config(&config_file).await;

    let output = match args.mode {
        BootstrapMode::Core | BootstrapMode::SetupL1 => {
            let output = setup_core_contract(&config_file, &clients).await;

            BootstrapperOutput {
                starknet_contract_address: Some(output.core_contract_client.address()),
                starknet_contract_implementation_address: Some(output.core_contract_client.implementation_address()),
                ..Default::default()
            }
        }
        BootstrapMode::SetupL2 => setup_l2(&config_file, &clients).await,
        BootstrapMode::EthBridge => {
            let core_contract_client = get_core_contract_client(&config_file, &clients);
            let output = setup_eth_bridge(None, &core_contract_client, &config_file, &clients).await;
            BootstrapperOutput { eth_bridge_setup_outputs: Some(output), ..Default::default() }
        }
        BootstrapMode::Erc20Bridge => {
            let core_contract_client = get_core_contract_client(&config_file, &clients);
            let output = setup_erc20_bridge(None, &core_contract_client, &config_file, &clients).await;
            BootstrapperOutput { erc20_bridge_setup_outputs: Some(output), ..Default::default() }
        }
        BootstrapMode::Udc => {
            let output = setup_udc(None, &config_file, &clients).await;
            BootstrapperOutput { udc_setup_outputs: Some(output), ..Default::default() }
        }
        BootstrapMode::Argent => {
            let output = setup_argent(None, &config_file, &clients).await;
            BootstrapperOutput { argent_setup_outputs: Some(output), ..Default::default() }
        }
        BootstrapMode::Braavos => {
            let output = setup_braavos(None, &config_file, &clients).await;
            BootstrapperOutput { braavos_setup_outputs: Some(output), ..Default::default() }
        }
    };

    let output_json =
        serde_json::to_string_pretty(&output).unwrap_or_else(|e| format!("Error serializing output: {}", e));

    // Print the output to the console
    println!("Bootstrap Output:");
    println!("{}", output_json);

    if let Some(output_file) = args.output_file {
        let file = File::create(&output_file).unwrap();
        serde_json::to_writer_pretty(file, &output).unwrap();
        println!("✅ Bootstrap output saved to {}", output_file);
    }
}

fn get_core_contract_client(config_file: &ConfigFile, clients: &Clients) -> CoreContractStarknetL1Output {
    let Some(core_contract_address) = config_file.core_contract_address.clone() else {
        panic!("Core contract address is required for ETH bridge setup");
    };
    let Some(core_contract_implementation_address) = config_file.core_contract_implementation_address.clone() else {
        panic!("Core contract implementation address is required for ETH bridge setup");
    };
    let core_contract_client = StarknetValidityContractClient::new(
        Address::from_str(&core_contract_address).unwrap(),
        clients.eth_client().signer().clone(),
        Address::from_str(&core_contract_implementation_address).unwrap(),
    );
    CoreContractStarknetL1Output { core_contract_client: Box::new(StarknetValidityContract { core_contract_client }) }
}

async fn get_account<'a>(clients: &'a Clients, config_file: &'a ConfigFile) -> RpcAccount<'a> {
    log::info!("⏳ L2 State and Initialisation Started");
    let account = account_init(clients, config_file).await;
    log::info!("🔐 Account with given  private key deployed on L2. [Account Address : {:?}]", account.address());
    account
}

#[derive(Serialize, Default)]
pub struct BootstrapperOutput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub starknet_contract_address: Option<Address>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub starknet_contract_implementation_address: Option<Address>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eth_bridge_setup_outputs: Option<EthBridgeSetupOutput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub erc20_bridge_setup_outputs: Option<Erc20BridgeSetupOutput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub udc_setup_outputs: Option<UdcSetupOutput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub argent_setup_outputs: Option<ArgentSetupOutput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub braavos_setup_outputs: Option<BraavosSetupOutput>,
}

pub async fn bootstrap(config_file: &ConfigFile, clients: &Clients) -> BootstrapperOutput {
    // setup core contract (L1)
    let core_contract_client = setup_core_contract(config_file, clients).await;

    // setup L2
    let l2_output = setup_l2(config_file, clients).await;

    BootstrapperOutput {
        starknet_contract_address: Some(core_contract_client.core_contract_client.address()),
        starknet_contract_implementation_address: Some(
            core_contract_client.core_contract_client.implementation_address(),
        ),
        ..l2_output
    }
}

async fn setup_core_contract(config_file: &ConfigFile, clients: &Clients) -> CoreContractStarknetL1Output {
    let core_contract = CoreContractStarknetL1::new(config_file, clients);
    let core_contract_client = core_contract.setup().await;
    log::info!("📦 Core address : {:?}", core_contract_client.core_contract_client.address());
    log::info!(
        "📦 Core implementation address : {:?}",
        core_contract_client.core_contract_client.implementation_address()
    );
    save_to_json(
        "l1_core_contract_address",
        &JsonValueType::EthAddress(core_contract_client.core_contract_client.address()),
    )
    .unwrap();
    log::info!("✅ Core setup init for L1 successful.");
    core_contract_client
}

async fn setup_eth_bridge<'a>(
    account: Option<RpcAccount<'a>>,
    core_contract_client: &CoreContractStarknetL1Output,
    config_file: &ConfigFile,
    clients: &Clients,
) -> EthBridgeSetupOutput {
    let account = match account {
        Some(account) => account,
        None => get_account(clients, config_file).await,
    };
    log::info!("⏳ Starting ETH bridge deployment");
    let eth_bridge = EthBridge::new(
        account.clone(),
        account.address(),
        config_file,
        clients,
        core_contract_client.core_contract_client.as_ref(),
    );
    let eth_bridge_setup_outputs = eth_bridge.setup().await;
    log::info!("✅ ETH bridge deployment complete.");
    eth_bridge_setup_outputs
}

async fn setup_erc20_bridge<'a>(
    account: Option<RpcAccount<'a>>,
    core_contract_client: &CoreContractStarknetL1Output,
    config_file: &ConfigFile,
    clients: &Clients,
) -> Erc20BridgeSetupOutput {
    let account = match account {
        Some(account) => account,
        None => get_account(clients, config_file).await,
    };
    log::info!("⏳ Starting ERC20 token bridge deployment");
    let erc20_bridge = Erc20Bridge::new(
        account.clone(),
        account.address(),
        config_file,
        clients,
        core_contract_client.core_contract_client.as_ref(),
    );
    let erc20_bridge_setup_outputs = erc20_bridge.setup().await;
    log::info!("✅ ERC20 token bridge deployment complete.");
    erc20_bridge_setup_outputs
}

async fn setup_udc<'a>(account: Option<RpcAccount<'a>>, config_file: &ConfigFile, clients: &Clients) -> UdcSetupOutput {
    let account = match account {
        Some(account) => account,
        None => get_account(clients, config_file).await,
    };
    log::info!("⏳ Starting UDC (Universal Deployer Contract) deployment");
    let udc = UdcSetup::new(account.clone(), account.address(), config_file, clients);
    let udc_setup_outputs = udc.setup().await;
    log::info!(
        "*️⃣ UDC setup completed. [UDC Address : {:?}, UDC class hash : {:?}]",
        udc_setup_outputs.udc_address,
        udc_setup_outputs.udc_class_hash
    );
    log::info!("✅ UDC (Universal Deployer Contract) deployment complete.");
    udc_setup_outputs
}

async fn setup_argent<'a>(
    account: Option<RpcAccount<'a>>,
    config_file: &ConfigFile,
    clients: &Clients,
) -> ArgentSetupOutput {
    let account = match account {
        Some(account) => account,
        None => get_account(clients, config_file).await,
    };
    log::info!("⏳ Starting Argent Account deployment");
    let argent = ArgentSetup::new(account.clone());
    let argent_setup_outputs = argent.setup().await;
    log::info!("*️⃣ Argent setup completed. [Argent account class hash : {:?}]", argent_setup_outputs.argent_class_hash);
    log::info!("✅ Argent Account deployment complete.");
    argent_setup_outputs
}

async fn setup_braavos<'a>(
    account: Option<RpcAccount<'a>>,
    config_file: &ConfigFile,
    clients: &Clients,
) -> BraavosSetupOutput {
    let account = match account {
        Some(account) => account,
        None => get_account(clients, config_file).await,
    };
    log::info!("⏳ Starting Braavos Account deployment");
    let braavos = BraavosSetup::new(account.clone(), config_file, clients);
    let braavos_setup_outputs = braavos.setup().await;
    log::info!(
        "*️⃣ Braavos setup completed. [Braavos account class hash : {:?}]",
        braavos_setup_outputs.braavos_class_hash
    );
    log::info!("✅ Braavos Account deployment complete.");
    braavos_setup_outputs
}

pub async fn setup_l2(config_file: &ConfigFile, clients: &Clients) -> BootstrapperOutput {
    let account = get_account(clients, config_file).await;

    let core_contract_client = get_core_contract_client(config_file, clients);

    // setup eth bridge
    let eth_bridge_setup_outputs =
        setup_eth_bridge(Some(account.clone()), &core_contract_client, config_file, clients).await;

    // setup erc20 bridge
    let erc20_bridge_setup_outputs =
        setup_erc20_bridge(Some(account.clone()), &core_contract_client, config_file, clients).await;

    // setup udc
    let udc_setup_outputs = setup_udc(Some(account.clone()), config_file, clients).await;

    // setup argent account
    let argent_setup_outputs = setup_argent(Some(account.clone()), config_file, clients).await;

    // setup braavos account
    let braavos_setup_outputs = setup_braavos(Some(account.clone()), config_file, clients).await;

    BootstrapperOutput {
        eth_bridge_setup_outputs: Some(eth_bridge_setup_outputs),
        erc20_bridge_setup_outputs: Some(erc20_bridge_setup_outputs),
        udc_setup_outputs: Some(udc_setup_outputs),
        argent_setup_outputs: Some(argent_setup_outputs),
        braavos_setup_outputs: Some(braavos_setup_outputs),
        ..Default::default()
    }
}
