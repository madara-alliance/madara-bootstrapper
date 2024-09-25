use std::sync::Arc;

use anyhow::Context;
use async_trait::async_trait;
use ethers::addressbook::Address;
use ethers::providers::Middleware;
use ethers::types::{Bytes, U256};
use starknet_accounts::{Account, ConnectedAccount};
use starknet_eth_bridge_client::clients::eth_bridge::StarknetEthBridgeContractClient;
use starknet_eth_bridge_client::interfaces::eth_bridge::StarknetEthBridgeTrait;
use starknet_eth_bridge_client::{
    deploy_starknet_eth_bridge_behind_safe_proxy, deploy_starknet_eth_bridge_behind_unsafe_proxy,
};
use starknet_ff::FieldElement;
use starknet_providers::jsonrpc::HttpTransport;
use starknet_providers::JsonRpcClient;
use starknet_proxy_client::interfaces::proxy::ProxySupport3_0_2Trait;
use zaun_utils::{LocalWalletSignerMiddleware, StarknetContractClient};

use crate::contract_clients::utils::{field_element_to_u256, RpcAccount};
use crate::helpers::account_actions::{get_contract_address_from_deploy_tx, AccountActions};
use crate::utils::{invoke_contract, wait_for_transaction};

#[async_trait]
pub trait BridgeDeployable: Sized {
    async fn deploy(client: Arc<LocalWalletSignerMiddleware>, is_dev: bool) -> anyhow::Result<Self>;
}

pub struct StarknetLegacyEthBridge {
    eth_bridge: StarknetEthBridgeContractClient,
}

#[async_trait]
impl BridgeDeployable for StarknetLegacyEthBridge {
    async fn deploy(client: Arc<LocalWalletSignerMiddleware>, is_dev: bool) -> anyhow::Result<Self> {
        let eth_bridge = match is_dev {
            true => deploy_starknet_eth_bridge_behind_unsafe_proxy(client.clone())
                .await
                .context("Failed to deploy starknet contract")?,
            false => deploy_starknet_eth_bridge_behind_safe_proxy(client.clone())
                .await
                .context("Failed to deploy starknet contract")?,
        };

        Ok(Self { eth_bridge })
    }
}

impl StarknetLegacyEthBridge {
    pub fn address(&self) -> Address {
        self.eth_bridge.address()
    }

    pub fn implementation_address(&self) -> Address {
        self.eth_bridge.implementation_address()
    }

    pub fn client(&self) -> Arc<LocalWalletSignerMiddleware> {
        self.eth_bridge.client()
    }

    pub async fn deploy_l2_contracts(
        rpc_provider_l2: &JsonRpcClient<HttpTransport>,
        legacy_eth_bridge_class_hash: FieldElement,
        legacy_eth_bridge_proxy_address: FieldElement,
        account: &RpcAccount<'_>,
    ) -> anyhow::Result<FieldElement> {
        let deploy_tx = account
            .invoke_contract(
                account.address(),
                "deploy_contract",
                vec![legacy_eth_bridge_class_hash, FieldElement::ZERO, FieldElement::ZERO, FieldElement::ZERO],
                None,
            )
            .context("Creating the invoke transaction for contract proxy deployment")?
            .send()
            .await
            .context("Error deploying the contract proxy")?;
        wait_for_transaction(
            rpc_provider_l2,
            deploy_tx.transaction_hash,
            "deploy_l2_contracts : deploy_contract : eth bridge",
        )
        .await
        .context("Waiting for contract proxy deployment")?;
        let contract_address = get_contract_address_from_deploy_tx(account.provider(), &deploy_tx)
            .await
            .context("Getting contract address from deploy transaction")?;

        log::debug!("🎡 contract address (eth bridge) : {:?}", contract_address);

        let add_implementation_txn = invoke_contract(
            legacy_eth_bridge_proxy_address,
            "add_implementation",
            vec![contract_address, FieldElement::ZERO, FieldElement::ONE, account.address(), FieldElement::ZERO],
            account,
        )
        .await
        .context("Creating the add_implementation transaction")?;

        wait_for_transaction(
            rpc_provider_l2,
            add_implementation_txn.transaction_hash,
            "deploy_l2_contracts : add_implementation : eth bridge",
        )
        .await
        .context("Waiting for the add_implementation transaction to settle")?;

        let upgrade_to_txn = invoke_contract(
            legacy_eth_bridge_proxy_address,
            "upgrade_to",
            vec![contract_address, FieldElement::ZERO, FieldElement::ONE, account.address(), FieldElement::ZERO],
            account,
        )
        .await
        .context("Creating the upgrade_to transaction")?;

        wait_for_transaction(
            rpc_provider_l2,
            upgrade_to_txn.transaction_hash,
            "deploy_l2_contracts : upgrade_to : eth bridge",
        )
        .await
        .context("Waiting for the upgrade_to transaction to settle")?;

        Ok(legacy_eth_bridge_proxy_address)
    }

    /// Initialize Starknet Legacy Eth Bridge
    /// IMP : only need to be called when using unsafe proxy
    pub async fn initialize(&self, messaging_contract: Address) -> anyhow::Result<()> {
        let empty_bytes = [0u8; 32];

        let messaging_bytes = messaging_contract.as_bytes();

        let mut padded_messaging_bytes = Vec::with_capacity(32);
        padded_messaging_bytes.extend(vec![0u8; 32 - messaging_bytes.len()]);
        padded_messaging_bytes.extend_from_slice(messaging_bytes);

        let mut calldata = Vec::new();
        calldata.extend(empty_bytes);
        calldata.extend(empty_bytes);
        calldata.extend(padded_messaging_bytes);

        self.eth_bridge.initialize(Bytes::from(calldata)).await.context("Failed to initialize eth bridge")?;
        Ok(())
    }

    /// Add Implementation Starknet Legacy Eth Bridge
    pub async fn add_implementation_eth_bridge(&self, messaging_contract: Address) -> anyhow::Result<()> {
        let empty_bytes = [0u8; 32];

        let messaging_bytes = messaging_contract.as_bytes();

        let mut padded_messaging_bytes = Vec::with_capacity(32);
        padded_messaging_bytes.extend(vec![0u8; 32 - messaging_bytes.len()]);
        padded_messaging_bytes.extend_from_slice(messaging_bytes);

        let mut calldata = Vec::new();
        // `empty_bytes` act as an empty params for the calldata we are passing in bytes.
        // Here in this case of ETH Bridge it represents the EIC contract address, Token Address (ETH)
        // EIC = 0x0000000000000000000000000000000000000000
        // ETH Address to be passed in bridge = 0x0000000000000000000000000000000000000000
        calldata.extend(empty_bytes);
        calldata.extend(empty_bytes);
        calldata.extend(padded_messaging_bytes);

        log::debug!("🎡 add_implementation_eth_bridge : bytes : {:?}", Bytes::from(calldata.clone()));

        self.eth_bridge
            .add_implementation(Bytes::from(calldata), self.implementation_address(), false)
            .await
            .context("Failed to initialize eth bridge")?;
        Ok(())
    }

    /// Upgrade To Starknet Legacy Eth Bridge
    pub async fn upgrade_to_eth_bridge(&self, messaging_contract: Address) -> anyhow::Result<()> {
        let empty_bytes = [0u8; 32];

        let messaging_bytes = messaging_contract.as_bytes();

        let mut padded_messaging_bytes = Vec::with_capacity(32);
        padded_messaging_bytes.extend(vec![0u8; 32 - messaging_bytes.len()]);
        padded_messaging_bytes.extend_from_slice(messaging_bytes);

        let mut calldata = Vec::new();
        // `empty_bytes` act as an empty params for the calldata we are passing in bytes.
        // Here in this case of ETH Bridge it represents the EIC contract address, Token Address (ETH)
        // EIC = 0x0000000000000000000000000000000000000000
        // ETH Address to be passed in bridge = 0x0000000000000000000000000000000000000000
        calldata.extend(empty_bytes);
        calldata.extend(empty_bytes);
        calldata.extend(padded_messaging_bytes);

        log::debug!("🎡 upgrade_to_eth_bridge : bytes : {:?}", Bytes::from(calldata.clone()));

        self.eth_bridge
            .upgrade_to(Bytes::from(calldata), self.implementation_address(), false)
            .await
            .context("Failed to upgrade the eth bridge")?;

        Ok(())
    }

    /// Sets up the Eth bridge with the specified data
    pub async fn setup_l1_bridge(
        &self,
        max_total_balance: &str,
        max_deposit: &str,
        l2_bridge: FieldElement,
        l1_multisig_address: Address,
        is_dev: bool,
    ) -> anyhow::Result<()> {
        self.eth_bridge
            .set_max_total_balance(
                U256::from_dec_str(max_total_balance).context("Converting max total balance to U256")?,
            )
            .await
            .context("Setting max total balance")?;
        self.eth_bridge
            .set_max_deposit(U256::from_dec_str(max_deposit).context("Converting max deposit to U256")?)
            .await
            .context("Setting the max deposit")?;
        self.eth_bridge
            .set_l2_token_bridge(field_element_to_u256(l2_bridge))
            .await
            .context("Setting the L2 token bridge")?;

        if !is_dev {
            // Nominating a new governor as l1 multi sig address
            self.eth_bridge
                .proxy_nominate_new_governor(l1_multisig_address)
                .await
                .context("Nominating the proxy governor")?;
        }
        Ok(())
    }

    pub async fn setup_l2_bridge(
        &self,
        rpc_provider: &JsonRpcClient<HttpTransport>,
        l2_bridge_address: FieldElement,
        erc20_address: FieldElement,
        l2_deployer_address: &str,
        account: &RpcAccount<'_>,
    ) -> anyhow::Result<()> {
        let tx = invoke_contract(
            l2_bridge_address,
            "initialize",
            vec![
                FieldElement::from_dec_str("1").expect("Converting a constant"),
                FieldElement::from_hex_be(l2_deployer_address).context("Parsing the L2 deployer address")?,
            ],
            account,
        )
        .await
        .context("Creating the initialize transaction")?;

        log::debug!("🎡 setup_l2_bridge : l2 bridge initialized //");
        wait_for_transaction(rpc_provider, tx.transaction_hash, "setup_l2_bridge : initialize")
            .await
            .context("Waiting for the initialize transaction to settle")?;

        let tx = invoke_contract(l2_bridge_address, "set_l2_token", vec![erc20_address], account)
            .await
            .context("Creating the set_l2_token transaction")?;

        log::debug!("🎡 setup_l2_bridge : l2 token set //");
        wait_for_transaction(rpc_provider, tx.transaction_hash, "setup_l2_bridge : set_l2_token")
            .await
            .context("Waiting for the set_l2_token transaction to settle")?;

        let tx = invoke_contract(
            l2_bridge_address,
            "set_l1_bridge",
            vec![
                FieldElement::from_byte_slice_be(self.eth_bridge.address().as_bytes())
                    .context("Parsing the eth_bridge address")?,
            ],
            account,
        )
        .await
        .context("Creating the set_l1_bridge transaction")?;

        log::debug!("🎡 setup_l2_bridge : l1 bridge set //");
        wait_for_transaction(rpc_provider, tx.transaction_hash, "setup_l2_bridge : set_l1_bridge")
            .await
            .context("Waiting for the set_l1_bridge transaction to settle")?;

        Ok(())
    }

    pub async fn set_max_total_balance(&self, amount: U256) -> anyhow::Result<()> {
        self.eth_bridge.set_max_total_balance(amount).await.context("Setting the max total balance")?;
        Ok(())
    }

    pub async fn set_max_deposit(&self, amount: U256) -> anyhow::Result<()> {
        self.eth_bridge.set_max_deposit(amount).await.context("Failed to set max deposit value in eth bridge")?;
        Ok(())
    }

    pub async fn set_l2_token_bridge(&self, l2_bridge: U256) -> anyhow::Result<()> {
        self.eth_bridge.set_l2_token_bridge(l2_bridge).await.context("Failed to set l2 bridge in eth bridge")?;
        Ok(())
    }

    pub async fn deposit(&self, amount: U256, l2_address: U256, fee: U256) -> anyhow::Result<()> {
        self.eth_bridge.deposit(amount, l2_address, fee).await.context("Failed to deposit in eth bridge")?;
        Ok(())
    }

    pub async fn withdraw(&self, amount: U256, l1_recipient: Address) -> anyhow::Result<()> {
        self.eth_bridge.withdraw(amount, l1_recipient).await.context("Failed to withdraw from eth bridge")?;
        Ok(())
    }

    pub async fn eth_balance(&self, l1_recipient: Address) -> anyhow::Result<U256> {
        let provider = self.eth_bridge.client().provider().clone();

        provider
            .get_balance(l1_recipient, None)
            .await
            .with_context(|| format!("Getting the eth balance for {}", l1_recipient))
    }
}
