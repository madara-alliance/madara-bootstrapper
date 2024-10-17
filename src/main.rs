pub mod contract_clients;
pub mod helpers;
mod setup_scripts;
#[cfg(test)]
pub mod tests;
pub mod utils;

use std::fs::File;
use std::io::stdin;
use std::str::FromStr;
use std::time::Duration;

use clap::{ArgAction, Parser, ValueEnum};
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
use starknet::core::types::Felt;
use starknet_core_contract_client::clients::StarknetValidityContractClient;
use tokio::time::sleep;

use crate::contract_clients::config::Config;
use crate::contract_clients::core_contract::CoreContract;
use crate::contract_clients::eth_bridge::StarknetLegacyEthBridge;
use crate::contract_clients::starknet_validity::StarknetValidityContract;
use crate::contract_clients::token_bridge::StarknetTokenBridge;
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
    Full,
    Core,
    EthBridge,
    Erc20Bridge,
    Udc,
    Argent,
    Braavos,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    #[clap(long, env, default_value = "http://127.0.0.1:8545")]
    eth_rpc: String,
    #[clap(long, env, default_value = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80")]
    eth_priv_key: String,
    #[clap(long, env, default_value = "http://127.0.0.1:9944")]
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
    #[clap(long, env, default_value = "MADARA_DEVNET")]
    app_chain_id: String,
    #[clap(long, env, default_value = "0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7")]
    fee_token_address: String,
    #[clap(long, env, default_value = "0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d")]
    native_fee_token_address: String,
    #[clap(long, env, default_value_t = 20)]
    cross_chain_wait_time: u64,
    // Default test address value taken from anvil
    // IMP : Not to be used in prod environment
    #[clap(long, env, default_value = "0x70997970C51812dc3A010C7d01b50e0d17dc79C8")]
    l1_multisig_address: String,
    // Default test address value taken from starknet-devnet
    // IMP : Not to be used in prod environment
    #[clap(long, env, default_value = "0x556455b8ac8bc00e0ad061d7df5458fa3c372304877663fa21d492a8d5e9435")]
    l2_multisig_address: String,
    // Given as 0xabcd by default
    #[clap(long, env, default_value = "0x000000000000000000000000000000000000abcd")]
    verifier_address: String,
    // Given as 0xabcd by default
    #[clap(long, env, default_value = "0x000000000000000000000000000000000000abcd")]
    operator_address: String,
    #[clap(long, env, action=ArgAction::SetTrue)]
    dev: bool,
    #[clap(long, env, value_enum, default_value_t = BootstrapMode::Full)]
    mode: BootstrapMode,
    #[clap(long, env)]
    core_contract_address: Option<String>,
    #[clap(long, env)]
    core_contract_implementation_address: Option<String>,
    #[clap(long, env)]
    output_file: Option<String>,
}

#[tokio::main]
pub async fn main() {
    env_logger::init();
    dotenv().ok();

    let args = CliArgs::parse();

    println!("{color_red}{}{color_reset}", BANNER);

    let config = Config::init(&args).await;

    let output = match args.mode {
        BootstrapMode::Full => bootstrap(&args, &config).await,
        BootstrapMode::Core => {
            let output = setup_core_contract(&args, &config).await;
            BootstrapperOutput {
                starknet_contract_address: Some(output.core_contract_client.address()),
                starknet_contract_implementation_address: Some(output.core_contract_client.implementation_address()),
                ..Default::default()
            }
        }
        BootstrapMode::EthBridge => {
            let core_contract_client = get_core_contract_client(&args, &config);
            let output = setup_eth_bridge(None, &core_contract_client, &args, &config).await;
            BootstrapperOutput { eth_bridge_setup_outputs: Some(output), ..Default::default() }
        }
        BootstrapMode::Erc20Bridge => {
            let core_contract_client = get_core_contract_client(&args, &config);
            let output = setup_erc20_bridge(None, &core_contract_client, &args, &config).await;
            BootstrapperOutput { erc20_bridge_setup_outputs: Some(output), ..Default::default() }
        }
        BootstrapMode::Udc => {
            let output = setup_udc(None, &args, &config).await;
            BootstrapperOutput { udc_setup_outputs: Some(output), ..Default::default() }
        }
        BootstrapMode::Argent => {
            let output = setup_argent(None, &args, &config).await;
            BootstrapperOutput { argent_setup_outputs: Some(output), ..Default::default() }
        }
        BootstrapMode::Braavos => {
            let output = setup_braavos(None, &args, &config).await;
            BootstrapperOutput { braavos_setup_outputs: Some(output), ..Default::default() }
        }
    };

    if let Some(output_file) = args.output_file {
        let file = File::create(&output_file).unwrap();
        serde_json::to_writer_pretty(file, &output).unwrap();
        println!("✅ Bootstrap output saved to {}", output_file);
    }
}

fn get_core_contract_client(args: &CliArgs, config: &Config) -> CoreContractStarknetL1Output {
    let Some(core_contract_address) = args.core_contract_address.clone() else {
        panic!("Core contract address is required for ETH bridge setup");
    };
    let Some(core_contract_implementation_address) = args.core_contract_implementation_address.clone() else {
        panic!("Core contract implementation address is required for ETH bridge setup");
    };
    let core_contract_client = StarknetValidityContractClient::new(
        Address::from_str(&core_contract_address).unwrap(),
        config.eth_client().signer().clone(),
        Address::from_str(&core_contract_implementation_address).unwrap(),
    );
    let core_contract_client = CoreContractStarknetL1Output {
        core_contract_client: Box::new(StarknetValidityContract { core_contract_client }),
    };

    core_contract_client
}

async fn get_account<'a>(config: &'a Config, args: &'a CliArgs) -> RpcAccount<'a> {
    log::info!("⏳ L2 State and Initialisation Started");
    let account = account_init(config, &args).await;
    log::info!("🔐 Account with given  private key deployed on L2. [Account Address : {:?}]", account.address());
    account
}

#[derive(Debug, Clone, Serialize, Default)]
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

pub async fn bootstrap(args: &CliArgs, config: &Config) -> BootstrapperOutput {
    let account = get_account(config, args).await;

    // setup core contract
    let core_contract_client = setup_core_contract(&args, config).await;

    // setup eth bridge
    let eth_bridge_setup_outputs = setup_eth_bridge(Some(account.clone()), &core_contract_client, args, config).await;

    // setup erc20 bridge
    let erc20_bridge_setup_outputs =
        setup_erc20_bridge(Some(account.clone()), &core_contract_client, args, config).await;

    // setup udc
    let udc_setup_outputs = setup_udc(Some(account.clone()), args, config).await;

    // setup argent account
    let argent_setup_outputs = setup_argent(Some(account.clone()), args, config).await;

    // setup braavos account
    let braavos_setup_outputs = setup_braavos(Some(account.clone()), args, config).await;

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
    }
}

async fn setup_core_contract(args: &CliArgs, config: &Config) -> CoreContractStarknetL1Output {
    let core_contract = CoreContractStarknetL1::new(args, &config);
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
    args: &CliArgs,
    config: &Config,
) -> EthBridgeSetupOutput {
    let account = match account {
        Some(account) => account,
        None => get_account(config, args).await,
    };
    log::info!("⏳ Starting ETH bridge deployment");
    let eth_bridge = EthBridge::new(
        account.clone(),
        account.address(),
        args,
        &config,
        core_contract_client.core_contract_client.as_ref(),
    );
    let eth_bridge_setup_outputs = eth_bridge.setup().await;
    log::info!("✅ ETH bridge deployment complete.");
    eth_bridge_setup_outputs
}

async fn setup_erc20_bridge<'a>(
    account: Option<RpcAccount<'a>>,
    core_contract_client: &CoreContractStarknetL1Output,
    args: &CliArgs,
    config: &Config,
) -> Erc20BridgeSetupOutput {
    let account = match account {
        Some(account) => account,
        None => get_account(config, args).await,
    };
    log::info!("⏳ Starting ERC20 token bridge deployment");
    let erc20_bridge = Erc20Bridge::new(
        account.clone(),
        account.address(),
        args,
        &config,
        core_contract_client.core_contract_client.as_ref(),
    );
    let erc20_bridge_setup_outputs = erc20_bridge.setup().await;
    log::info!("✅ ERC20 token bridge deployment complete.");
    erc20_bridge_setup_outputs
}

async fn setup_udc<'a>(account: Option<RpcAccount<'a>>, args: &CliArgs, config: &Config) -> UdcSetupOutput {
    let account = match account {
        Some(account) => account,
        None => get_account(config, args).await,
    };
    log::info!("⏳ Starting UDC (Universal Deployer Contract) deployment");
    let udc = UdcSetup::new(account.clone(), account.address(), args);
    let udc_setup_outputs = udc.setup().await;
    log::info!(
        "*️⃣ UDC setup completed. [UDC Address : {:?}, UDC class hash : {:?}]",
        udc_setup_outputs.udc_address,
        udc_setup_outputs.udc_class_hash
    );
    log::info!("✅ UDC (Universal Deployer Contract) deployment complete.");
    udc_setup_outputs
}

async fn setup_argent<'a>(account: Option<RpcAccount<'a>>, args: &CliArgs, config: &Config) -> ArgentSetupOutput {
    let account = match account {
        Some(account) => account,
        None => get_account(config, args).await,
    };
    log::info!("⏳ Starting Argent Account deployment");
    let argent = ArgentSetup::new(account.clone());
    let argent_setup_outputs = argent.setup().await;
    log::info!("*️⃣ Argent setup completed. [Argent account class hash : {:?}]", argent_setup_outputs.argent_class_hash);
    log::info!("✅ Argent Account deployment complete.");
    argent_setup_outputs
}

async fn setup_braavos<'a>(account: Option<RpcAccount<'a>>, args: &CliArgs, config: &Config) -> BraavosSetupOutput {
    let account = match account {
        Some(account) => account,
        None => get_account(config, args).await,
    };
    log::info!("⏳ Starting Braavos Account deployment");
    let braavos = BraavosSetup::new(account.clone(), args);
    let braavos_setup_outputs = braavos.setup().await;
    log::info!(
        "*️⃣ Braavos setup completed. [Braavos account class hash : {:?}]",
        braavos_setup_outputs.braavos_class_hash
    );
    log::info!("✅ Braavos Account deployment complete.");
    braavos_setup_outputs
}
