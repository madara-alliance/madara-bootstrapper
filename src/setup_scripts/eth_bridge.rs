use std::str::FromStr;
use std::time::Duration;

use anyhow::Context;
use ethers::abi::Address;
use starknet_accounts::{Account, ConnectedAccount};
use starknet_ff::FieldElement;
use starknet_providers::jsonrpc::HttpTransport;
use starknet_providers::JsonRpcClient;
use tokio::time::sleep;

use crate::contract_clients::config::Config;
use crate::contract_clients::core_contract::CoreContract;
use crate::contract_clients::eth_bridge::{BridgeDeployable, StarknetLegacyEthBridge};
use crate::contract_clients::utils::{
    build_single_owner_account, declare_contract, deploy_proxy_contract, init_governance_proxy, DeclarationInput,
    RpcAccount,
};
use crate::helpers::account_actions::{get_contract_address_from_deploy_tx, AccountActions};
use crate::utils::constants::{ERC20_LEGACY_PATH, LEGACY_BRIDGE_PATH, PROXY_LEGACY_PATH, STARKGATE_PROXY_PATH};
use crate::utils::{convert_to_hex, invoke_contract, save_to_json, wait_for_transaction, JsonValueType};
use crate::CliArgs;

pub struct EthBridge<'a> {
    account: RpcAccount<'a>,
    account_address: FieldElement,
    arg_config: &'a CliArgs,
    clients: &'a Config,
    core_contract: &'a dyn CoreContract,
}

pub struct EthBridgeSetupOutput {
    pub legacy_proxy_class_hash: FieldElement,
    pub starkgate_proxy_class_hash: FieldElement,
    pub erc20_legacy_class_hash: FieldElement,
    pub legacy_eth_bridge_class_hash: FieldElement,
    pub eth_proxy_address: FieldElement,
    pub eth_bridge_proxy_address: FieldElement,
    pub eth_bridge: StarknetLegacyEthBridge,
}

impl<'a> EthBridge<'a> {
    pub fn new(
        account: RpcAccount<'a>,
        account_address: FieldElement,
        arg_config: &'a CliArgs,
        clients: &'a Config,
        core_contract: &'a dyn CoreContract,
    ) -> Self {
        Self { account, account_address, arg_config, clients, core_contract }
    }

    pub async fn setup(&self) -> anyhow::Result<EthBridgeSetupOutput> {
        let legacy_proxy_class_hash = declare_contract(DeclarationInput::LegacyDeclarationInputs(
            String::from(PROXY_LEGACY_PATH),
            self.arg_config.rollup_seq_url.clone(),
        ))
        .await
        .context("Declaring legacy proxy class")?;
        log::debug!("🎡 Legacy proxy class hash declared.");
        save_to_json("legacy_proxy_class_hash", &JsonValueType::StringType(legacy_proxy_class_hash.to_string()))
            .context("Saving legacy proxy class hash to json")?;
        sleep(Duration::from_secs(10)).await;

        let starkgate_proxy_class_hash = declare_contract(DeclarationInput::LegacyDeclarationInputs(
            String::from(STARKGATE_PROXY_PATH),
            self.arg_config.rollup_seq_url.clone(),
        ))
        .await
        .context("Declaring starkgate proxy class")?;
        log::debug!("🎡 Starkgate proxy class hash declared.");
        save_to_json("starkgate_proxy_class_hash", &JsonValueType::StringType(starkgate_proxy_class_hash.to_string()))
            .context("Saving starkget proxy class hash to json")?;
        sleep(Duration::from_secs(10)).await;

        let erc20_legacy_class_hash = declare_contract(DeclarationInput::LegacyDeclarationInputs(
            String::from(ERC20_LEGACY_PATH),
            self.arg_config.rollup_seq_url.clone(),
        ))
        .await
        .context("Declaring erc20 legacy class")?;
        log::debug!("🎡 ERC20 legacy class hash declared.");
        save_to_json("erc20_legacy_class_hash", &JsonValueType::StringType(erc20_legacy_class_hash.to_string()))
            .context("Saving erc20 legacy class hash to json")?;
        sleep(Duration::from_secs(10)).await;

        let legacy_eth_bridge_class_hash = declare_contract(DeclarationInput::LegacyDeclarationInputs(
            String::from(LEGACY_BRIDGE_PATH),
            self.arg_config.rollup_seq_url.clone(),
        ))
        .await
        .context("Declaring legacy eth bridge class")?;
        log::debug!("🎡 Legacy ETH Bridge class hash declared");
        save_to_json(
            "legacy_eth_bridge_class_hash",
            &JsonValueType::StringType(legacy_eth_bridge_class_hash.to_string()),
        )
        .context("Saving legacy eth bridge class hash to json")?;
        sleep(Duration::from_secs(10)).await;

        let eth_proxy_address = deploy_proxy_contract(
            &self.account,
            self.account_address,
            legacy_proxy_class_hash,
            // salt taken from : https://sepolia.starkscan.co/tx/0x06a5a493cf33919e58aa4c75777bffdef97c0e39cac968896d7bee8cc67905a1
            FieldElement::from_str("0x322c2610264639f6b2cee681ac53fa65c37e187ea24292d1b21d859c55e1a78")
                .expect("Parsing a constant"),
            FieldElement::ONE,
        )
        .await
        .context("Deploying ETH ERC20 proxy contract")?;
        log::info!("✴️ ETH ERC20 proxy deployed [ETH : {:?}]", eth_proxy_address);
        save_to_json("l2_eth_address_proxy", &JsonValueType::StringType(eth_proxy_address.to_string()))
            .context("Saving ETH ERC20 proxy contract address to json")?;
        sleep(Duration::from_secs(10)).await;

        let eth_bridge_proxy_address = deploy_proxy_contract(
            &self.account,
            self.account_address,
            legacy_proxy_class_hash,
            FieldElement::from_str("0xabcdabcdabcd").expect("Parsing constant"),
            FieldElement::ZERO,
        )
        .await
        .context("Deploying ETH brudge proxy contract")?;
        log::info!("✴️ ETH Bridge proxy deployed [ETH Bridge : {:?}]", eth_bridge_proxy_address);
        save_to_json("ETH_l2_bridge_address_proxy", &JsonValueType::StringType(eth_bridge_proxy_address.to_string()))
            .context("Saving ETH brudge proxy contract to json")?;
        sleep(Duration::from_secs(10)).await;

        init_governance_proxy(&self.account, eth_proxy_address, "eth_proxy_address : init_governance_proxy")
            .await
            .context("Initializing governance proxy")?;
        sleep(Duration::from_secs(10)).await;
        init_governance_proxy(
            &self.account,
            eth_bridge_proxy_address,
            "eth_bridge_proxy_address : init_governance_proxy",
        )
        .await
        .context("Initializing governance proxy")?;
        sleep(Duration::from_secs(10)).await;

        let eth_bridge = StarknetLegacyEthBridge::deploy(self.core_contract.client().clone(), self.arg_config.dev)
            .await
            .context("Declaring starknet legacy eth bridge contract")?;

        log::info!("✴️ ETH Bridge L1 deployment completed [Eth Bridge Address (L1) : {:?}]", eth_bridge.address());
        save_to_json("ETH_l1_bridge_address", &JsonValueType::EthAddress(eth_bridge.address()))
            .context("Saving l1 eth bridge address to json")?;

        let account = build_single_owner_account(
            self.clients.provider_l2(),
            &self.arg_config.rollup_priv_key,
            &convert_to_hex(&self.account_address.to_string())?,
            false,
        )
        .await
        .context("Building single owner account")?;

        let l2_bridge_address = StarknetLegacyEthBridge::deploy_l2_contracts(
            self.clients.provider_l2(),
            legacy_eth_bridge_class_hash,
            eth_bridge_proxy_address,
            &account,
        )
        .await
        .context("Deploying starknet legacy eth bridge l2 contracts")?;

        log::info!("✴️ ETH Bridge L2 deployment completed [Eth Bridge Address (L2) : {:?}]", l2_bridge_address);
        save_to_json("ETH_l2_bridge_address", &JsonValueType::StringType(l2_bridge_address.to_string()))
            .context("Saving ETH l2 bridge address to json")?;

        let eth_address = deploy_eth_token_on_l2(
            self.clients.provider_l2(),
            eth_proxy_address,
            erc20_legacy_class_hash,
            &account,
            l2_bridge_address,
        )
        .await
        .context("Deploying eth token on l2")?;

        log::info!("✴️ L2 ETH token deployment successful.");
        // save_to_json("l2_eth_address", &JsonValueType::StringType(eth_address.to_string()))?;
        if self.arg_config.dev {
            eth_bridge
                .initialize(self.core_contract.address())
                .await
                .context("Initializing eth bridge in dev mode on L1")?;
        } else {
            eth_bridge
                .add_implementation_eth_bridge(self.core_contract.address())
                .await
                .context("Adding implementation eth bridge on L1")?;
            eth_bridge
                .upgrade_to_eth_bridge(self.core_contract.address())
                .await
                .context("Upgrading eth bridge on L1")?;
        }
        log::info!("✴️ ETH Bridge initialization on L1 completed");

        sleep(Duration::from_secs(self.arg_config.l1_wait_time.parse().context("Parsing L1 wait time")?)).await;

        eth_bridge
            .setup_l2_bridge(
                self.clients.provider_l2(),
                l2_bridge_address,
                eth_address,
                &self.arg_config.rollup_priv_key,
                &account,
            )
            .await
            .context("Setting up ETH bridge on L2")?;
        log::info!("✴️ ETH Bridge initialization and setup on L2 completed");

        eth_bridge
            .setup_l1_bridge(
                "10000000000000000000000000000000000000000",
                "10000000000000000000000000000000000000000",
                l2_bridge_address,
                Address::from_str(&self.arg_config.l1_multisig_address.to_string())
                    .context("Parsing L1 multisig address")?,
                self.arg_config.dev,
            )
            .await
            .context("Setting up ETH bridge on L1")?;
        log::info!("✴️ ETH Bridge setup on L1 completed");

        Ok(EthBridgeSetupOutput {
            legacy_proxy_class_hash,
            starkgate_proxy_class_hash,
            erc20_legacy_class_hash,
            legacy_eth_bridge_class_hash,
            eth_proxy_address,
            eth_bridge_proxy_address,
            eth_bridge,
        })
    }
}

pub async fn deploy_eth_token_on_l2(
    rpc_provider_l2: &JsonRpcClient<HttpTransport>,
    eth_proxy_address: FieldElement,
    eth_erc20_class_hash: FieldElement,
    account: &RpcAccount<'_>,
    eth_legacy_bridge_address: FieldElement,
) -> anyhow::Result<FieldElement> {
    let deploy_tx = account
        .invoke_contract(
            account.address(),
            "deploy_contract",
            vec![eth_erc20_class_hash, FieldElement::ZERO, FieldElement::ZERO, FieldElement::ZERO],
            None,
        )
        .context("Making deploy_contract transaction")?
        .send()
        .await
        .context("Error deploying the contract proxy")?;
    wait_for_transaction(rpc_provider_l2, deploy_tx.transaction_hash, "deploy_eth_token_on_l2 : deploy")
        .await
        .context("Waiting for deploy_contract transaction to settle")?;
    let contract_address = get_contract_address_from_deploy_tx(account.provider(), &deploy_tx)
        .await
        .context("Getting resulting contract address")?;

    log::debug!("Contract address (eth erc20) : {:?}", contract_address);

    let add_implementation_txn = invoke_contract(
        eth_proxy_address,
        "add_implementation",
        vec![
            contract_address,
            FieldElement::ZERO,
            FieldElement::from(4u64),
            FieldElement::from_byte_slice_be("Ether".as_bytes()).expect("Parsing a constant"),
            FieldElement::from_byte_slice_be("ETH".as_bytes()).expect("Parsing a constant"),
            FieldElement::from_str("18").expect("Parsing a constant"),
            eth_legacy_bridge_address,
            FieldElement::ZERO,
        ],
        account,
    )
    .await
    .context("Invoking add_implementation")?;

    wait_for_transaction(
        rpc_provider_l2,
        add_implementation_txn.transaction_hash,
        "deploy_eth_token_on_l2 : add_implementation",
    )
    .await
    .context("Waiting for add_implementation transaction to settle")?;

    let upgrade_to_txn = invoke_contract(
        eth_proxy_address,
        "upgrade_to",
        vec![
            contract_address,
            FieldElement::ZERO,
            FieldElement::from(4u64),
            FieldElement::from_byte_slice_be("Ether".as_bytes()).expect("Parsing a constant"),
            FieldElement::from_byte_slice_be("ETH".as_bytes()).expect("Parsing a constant"),
            FieldElement::from_str("18").expect("Parsing a constant"),
            eth_legacy_bridge_address,
            FieldElement::ZERO,
        ],
        account,
    )
    .await
    .context("Invoking upgrade_to")?;

    wait_for_transaction(rpc_provider_l2, upgrade_to_txn.transaction_hash, "deploy_eth_token_on_l2 : upgrade_to")
        .await
        .context("Waiting for upgrade_to transaction to settle")?;
    Ok(eth_proxy_address)
}
