use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use async_trait::async_trait;
use ethers::addressbook::Address;
use ethers::prelude::U256;
use ethers::types::Bytes;
use starkgate_manager_client::clients::starkgate_manager::StarkgateManagerContractClient;
use starkgate_manager_client::interfaces::manager::StarkgateManagerTrait;
use starkgate_manager_client::{
    deploy_starkgate_manager_behind_safe_proxy, deploy_starkgate_manager_behind_unsafe_proxy,
};
use starkgate_registry_client::clients::starkgate_registry::StarkgateRegistryContractClient;
use starkgate_registry_client::{
    deploy_starkgate_registry_behind_safe_proxy, deploy_starkgate_registry_behind_unsafe_proxy,
};
use starknet_accounts::{Account, ConnectedAccount};
use starknet_erc20_client::clients::erc20::ERC20ContractClient;
use starknet_erc20_client::deploy_dai_erc20_behind_unsafe_proxy;
use starknet_erc20_client::interfaces::erc20::ERC20TokenTrait;
use starknet_ff::FieldElement;
use starknet_providers::jsonrpc::HttpTransport;
use starknet_providers::JsonRpcClient;
use starknet_proxy_client::interfaces::proxy::ProxySupport5_0_0Trait;
use starknet_token_bridge_client::clients::token_bridge::StarknetTokenBridgeContractClient;
use starknet_token_bridge_client::interfaces::token_bridge::StarknetTokenBridgeTrait;
use starknet_token_bridge_client::{
    deploy_starknet_token_bridge_behind_safe_proxy, deploy_starknet_token_bridge_behind_unsafe_proxy,
};
use tokio::time::sleep;
use zaun_utils::{LocalWalletSignerMiddleware, StarknetContractClient};

use crate::contract_clients::eth_bridge::BridgeDeployable;
use crate::contract_clients::utils::{
    build_single_owner_account, declare_contract, field_element_to_u256, DeclarationInput, RpcAccount,
};
use crate::helpers::account_actions::{get_contract_address_from_deploy_tx, AccountActions};
use crate::utils::constants::{TOKEN_BRIDGE_CASM_PATH, TOKEN_BRIDGE_SIERRA_PATH};
use crate::utils::{invoke_contract, pad_bytes, save_to_json, wait_for_transaction, JsonValueType};

pub struct StarknetTokenBridge {
    manager: StarkgateManagerContractClient,
    registry: StarkgateRegistryContractClient,
    token_bridge: StarknetTokenBridgeContractClient,
    erc20: ERC20ContractClient,
}

#[async_trait]
impl BridgeDeployable for StarknetTokenBridge {
    async fn deploy(client: Arc<LocalWalletSignerMiddleware>, is_dev: bool) -> anyhow::Result<Self> {
        let manager = match is_dev {
            false => deploy_starkgate_manager_behind_safe_proxy(client.clone())
                .await
                .context("Failed to deploy starkgate manager contract")?,
            true => deploy_starkgate_manager_behind_unsafe_proxy(client.clone())
                .await
                .context("Failed to deploy starkgate manager contract")?,
        };
        let registry = match is_dev {
            false => deploy_starkgate_registry_behind_safe_proxy(client.clone())
                .await
                .context("Failed to deploy starkgate registry")?,
            true => deploy_starkgate_registry_behind_unsafe_proxy(client.clone())
                .await
                .context("Failed to deploy starkgate registry")?,
        };
        let token_bridge = match is_dev {
            false => deploy_starknet_token_bridge_behind_safe_proxy(client.clone())
                .await
                .context("Failed to deploy starknet contract")?,
            true => deploy_starknet_token_bridge_behind_unsafe_proxy(client.clone())
                .await
                .context("Failed to deploy starknet contract")?,
        };

        let erc20 = deploy_dai_erc20_behind_unsafe_proxy(client.clone())
            .await
            .context("Failed to deploy DAI erc20 contract")?;

        Ok(Self { manager, registry, token_bridge, erc20 })
    }
}

impl StarknetTokenBridge {
    pub fn manager_address(&self) -> Address {
        self.manager.address()
    }
    pub fn registry_address(&self) -> Address {
        self.registry.address()
    }
    pub fn bridge_address(&self) -> Address {
        self.token_bridge.address()
    }
    pub fn address(&self) -> Address {
        self.erc20.address()
    }

    pub fn manager_client(&self) -> Arc<LocalWalletSignerMiddleware> {
        self.manager.client()
    }
    pub fn registry_client(&self) -> Arc<LocalWalletSignerMiddleware> {
        self.registry.client()
    }
    pub fn bridge_client(&self) -> Arc<LocalWalletSignerMiddleware> {
        self.token_bridge.client()
    }
    pub fn erc20_client(&self) -> Arc<LocalWalletSignerMiddleware> {
        self.erc20.client()
    }

    pub async fn deploy_l2_contracts(
        rpc_provider_l2: &JsonRpcClient<HttpTransport>,
        priv_key: &str,
        l2_deployer_address: &str,
    ) -> anyhow::Result<FieldElement> {
        let account = build_single_owner_account(rpc_provider_l2, priv_key, l2_deployer_address, false)
            .await
            .context("Making the single owner account")?;

        let token_bridge_class_hash = declare_contract(DeclarationInput::DeclarationInputs(
            String::from(TOKEN_BRIDGE_SIERRA_PATH),
            String::from(TOKEN_BRIDGE_CASM_PATH),
            account.clone(),
        ))
        .await
        .context("Declaring token bridge contract")?;

        sleep(Duration::from_secs(10)).await;
        save_to_json("L2_token_bridge_class_hash", &JsonValueType::StringType(token_bridge_class_hash.to_string()))
            .context("Saving the l2 token bridge class hash to json")?;

        log::debug!("🌗 token_bridge_class_hash : {:?}", token_bridge_class_hash);

        let deploy_contract_implementation_token_bridge = account
            .invoke_contract(
                account.address(),
                "deploy_contract",
                vec![
                    token_bridge_class_hash,
                    FieldElement::ZERO,
                    FieldElement::ZERO,
                    FieldElement::ONE,
                    FieldElement::ZERO,
                ],
                None,
            )
            .context("Making the deploy token bridge transaction")?
            .send()
            .await
            .context("Error deploying the l2 contract proxy")?;
        wait_for_transaction(
            account.provider(),
            deploy_contract_implementation_token_bridge.transaction_hash,
            "deploy_l2_contracts : deploy_contract : token bridge",
        )
        .await
        .context("Waiting for l2 contract proxy to be deployed")?;
        sleep(Duration::from_secs(10)).await;
        let address_token_bridge_impl =
            get_contract_address_from_deploy_tx(account.provider(), &deploy_contract_implementation_token_bridge)
                .await
                .context("Getting the token bridge contract address")?;
        log::debug!("🌗 contract address (token bridge) : {:?}", address_token_bridge_impl);

        Ok(address_token_bridge_impl)
    }

    /// Initialize Starknet Token Bridge.
    /// IMP : only need to be called when using unsafe proxy
    pub async fn initialize(&self, messaging_contract: Address, governor: Address) -> anyhow::Result<()> {
        let empty_bytes = [0u8; 32];

        let mut manager_calldata = Vec::new();
        manager_calldata.extend(empty_bytes);
        manager_calldata.extend(pad_bytes(self.registry_address()));
        manager_calldata.extend(pad_bytes(self.bridge_address()));

        let mut registry_calldata = Vec::new();
        registry_calldata.extend(empty_bytes);
        registry_calldata.extend(pad_bytes(self.manager_address()));

        let mut bridge_calldata = Vec::new();
        bridge_calldata.extend(empty_bytes);
        bridge_calldata.extend(pad_bytes(self.manager_address()));
        bridge_calldata.extend(pad_bytes(messaging_contract));

        self.manager
            .initialize(Bytes::from(manager_calldata))
            .await
            .context("Failed to initialize starkgate manager")?;
        self.registry
            .initialize(Bytes::from(registry_calldata))
            .await
            .context("Failed to initialize starkgate registry")?;
        self.token_bridge
            .initialize(Bytes::from(bridge_calldata))
            .await
            .context("Failed to initialize starknet token bridge")?;

        // registering app governor temporarily
        self.token_bridge.register_app_role_admin(governor).await.context("Registering bridge app role admin")?;
        self.token_bridge.register_app_governor(governor).await.context("Registering bridge app governor")?;

        Ok(())
    }

    /// Add Implementation Starknet Token Bridge
    pub async fn add_implementation_token_bridge(&self, messaging_contract: Address) -> anyhow::Result<()> {
        let empty_bytes = [0u8; 32];

        let mut manager_calldata = Vec::new();
        manager_calldata.extend(empty_bytes);
        manager_calldata.extend(pad_bytes(self.registry_address()));
        manager_calldata.extend(pad_bytes(self.bridge_address()));

        let mut registry_calldata = Vec::new();
        registry_calldata.extend(empty_bytes);
        registry_calldata.extend(pad_bytes(self.manager_address()));

        let mut bridge_calldata = Vec::new();
        bridge_calldata.extend(empty_bytes);
        bridge_calldata.extend(pad_bytes(self.manager_address()));
        bridge_calldata.extend(pad_bytes(messaging_contract));

        self.manager
            .add_implementation(Bytes::from(manager_calldata.clone()), self.manager.implementation_address(), false)
            .await
            .context("Failed to initialize starkgate manager")?;
        log::debug!("🎡 add_implementation_token_bridge : manager bytes : {:?}", Bytes::from(manager_calldata));
        self.registry
            .add_implementation(Bytes::from(registry_calldata.clone()), self.registry.implementation_address(), false)
            .await
            .context("Failed to initialize starkgate registry")?;
        log::debug!("🎡 add_implementation_token_bridge : registry bytes : {:?}", Bytes::from(registry_calldata));
        self.token_bridge
            .add_implementation(Bytes::from(bridge_calldata.clone()), self.token_bridge.implementation_address(), false)
            .await
            .context("Failed to initialize eth bridge")?;
        log::debug!("🎡 add_implementation_token_bridge : token_bridge bytes : {:?}", Bytes::from(bridge_calldata));
        Ok(())
    }

    /// Upgrade To Starknet Token Bridge
    pub async fn upgrade_to_token_bridge(&self, messaging_contract: Address) -> anyhow::Result<()> {
        let empty_bytes = [0u8; 32];

        let mut manager_calldata = Vec::new();
        manager_calldata.extend(empty_bytes);
        manager_calldata.extend(pad_bytes(self.registry_address()));
        manager_calldata.extend(pad_bytes(self.bridge_address()));

        let mut registry_calldata = Vec::new();
        registry_calldata.extend(empty_bytes);
        registry_calldata.extend(pad_bytes(self.manager_address()));

        let mut bridge_calldata = Vec::new();
        bridge_calldata.extend(empty_bytes);
        bridge_calldata.extend(pad_bytes(self.manager_address()));
        bridge_calldata.extend(pad_bytes(messaging_contract));

        self.manager
            .upgrade_to(Bytes::from(manager_calldata.clone()), self.manager.implementation_address(), false)
            .await
            .context("Failed to initialize starkgate manager")?;
        log::debug!("🎡 upgrade_to_token_bridge : manager bytes : {:?}", Bytes::from(manager_calldata));
        self.registry
            .upgrade_to(Bytes::from(registry_calldata.clone()), self.registry.implementation_address(), false)
            .await
            .context("Failed to initialize starkgate registry")?;
        log::debug!("🎡 upgrade_to_token_bridge : registry bytes : {:?}", Bytes::from(registry_calldata));
        self.token_bridge
            .upgrade_to(Bytes::from(bridge_calldata.clone()), self.token_bridge.implementation_address(), false)
            .await
            .context("Failed to initialize eth bridge")?;
        log::debug!("🎡 upgrade_to_token_bridge : token_bridge bytes : {:?}", Bytes::from(bridge_calldata));
        Ok(())
    }

    /// Sets up the Token bridge with the specified data
    pub async fn setup_permissions_with_bridge_l1(
        &self,
        governor: Address,
        l1_multisig_address: Address,
    ) -> anyhow::Result<()> {
        // Register roles
        self.token_bridge.register_app_governor(governor).await.context("Registering token bridge app governor")?;
        self.token_bridge.register_app_role_admin(governor).await.context("Registering token bridge app role admin")?;
        self.token_bridge.register_security_admin(governor).await.context("Registering token bridge security admin")?;
        self.token_bridge.register_security_agent(governor).await.context("Registering token security agent")?;

        // Nominating a new governor with l1_multisig_address
        self.token_bridge
            .register_app_governor(l1_multisig_address)
            .await
            .context("Registering token bridge app governor")?;
        self.manager.register_app_governor(l1_multisig_address).await.context("Registering manager app governor")?;
        self.registry.register_app_governor(l1_multisig_address).await.context("Registering registry app governor")?;
        self.token_bridge
            .register_app_role_admin(l1_multisig_address)
            .await
            .context("Registering token admin app role admin")?;
        self.manager
            .register_app_role_admin(l1_multisig_address)
            .await
            .context("Registering manager app role admin")?;
        self.registry
            .register_app_role_admin(l1_multisig_address)
            .await
            .context("Registering registry app role admin")?;
        Ok(())
    }

    /// Deploys a test ERC20 token from L1 to L2
    pub async fn setup_l1_bridge(&self, fee: U256, l2_bridge: FieldElement) -> anyhow::Result<()> {
        self.token_bridge
            .set_l2_token_bridge(field_element_to_u256(l2_bridge))
            .await
            .context("Setting l2 token bridge field")?;
        self.manager
            .enroll_token_bridge(self.address(), fee)
            .await
            .context("Failed to enroll token bridge in manager")?;
        Ok(())
    }

    pub async fn setup_l2_bridge(
        &self,
        rpc_provider_l2: &JsonRpcClient<HttpTransport>,
        l2_bridge: FieldElement,
        l2_address: &str,
        account: &RpcAccount<'_>,
        erc20_class_hash: FieldElement,
    ) -> anyhow::Result<()> {
        let l2_address = FieldElement::from_hex_be(l2_address).context("Parsing l2 address")?;
        let tx = invoke_contract(l2_bridge, "register_app_role_admin", vec![l2_address], account)
            .await
            .context("Making the register_app_role_admin transacction")?;

        wait_for_transaction(
            rpc_provider_l2,
            tx.transaction_hash,
            "setup_l2_bridge : token bridge : register_app_role_admin",
        )
        .await
        .context("Waiting for the register_app_role_admin transacction to settle")?;
        log::debug!("🌗 setup_l2_bridge : register_app_role_admin //");

        let tx = invoke_contract(l2_bridge, "register_app_governor", vec![l2_address], account)
            .await
            .context("Making the register_app_governor transaction")?;

        wait_for_transaction(
            rpc_provider_l2,
            tx.transaction_hash,
            "setup_l2_bridge : token bridge : register_app_governor",
        )
        .await
        .context("Waiting for the register_app_governor transaction to settle")?;
        log::debug!("🌗 setup_l2_bridge : register_app_governor //");

        let tx = invoke_contract(l2_bridge, "set_l2_token_governance", vec![l2_address], account)
            .await
            .context("Making the set_l2_token_governance transaction")?;

        wait_for_transaction(
            rpc_provider_l2,
            tx.transaction_hash,
            "setup_l2_bridge : token bridge : set_l2_token_governance",
        )
        .await
        .context("Waiting for the set_l2_token_governance transaction to settle")?;
        log::debug!("🌗 setup_l2_bridge : set_l2_token_governance //");

        let tx = invoke_contract(
            l2_bridge,
            "set_erc20_class_hash",
            vec![
                erc20_class_hash, // class hash
            ],
            account,
        )
        .await
        .context("Making the set_erc20_class_hash transaction")?;

        wait_for_transaction(
            rpc_provider_l2,
            tx.transaction_hash,
            "setup_l2_bridge : token bridge : set_erc20_class_hash",
        )
        .await
        .context("Waiting for the setup_l2_bridge transaction to settle")?;
        log::debug!("🌗 setup_l2_bridge : set_erc20_class_hash //");

        let tx = invoke_contract(
            l2_bridge,
            "set_l1_bridge",
            vec![
                FieldElement::from_byte_slice_be(self.token_bridge.address().as_bytes())
                    .context("Parsing token bridge address")?,
            ],
            account,
        )
        .await
        .context("Making the set_l1_bridge transaction")?;
        wait_for_transaction(rpc_provider_l2, tx.transaction_hash, "setup_l2_bridge : token bridge : set_l1_bridge")
            .await
            .context("Waiting for the set_l1_bridge transaction to settle")?;
        log::debug!("🌗 setup_l2_bridge : set_l1_bridge //");
        Ok(())
    }

    pub async fn register_app_role_admin(&self, address: Address) -> anyhow::Result<()> {
        self.token_bridge
            .register_app_role_admin(address)
            .await
            .context("Failed to register app role admin in starknet token bridge")?;
        Ok(())
    }

    pub async fn register_app_governor(&self, address: Address) -> anyhow::Result<()> {
        self.token_bridge
            .register_app_governor(address)
            .await
            .context("Failed to register app governor in starknet token bridge")?;
        Ok(())
    }

    pub async fn set_l2_token_bridge(&self, l2_bridge: U256) -> anyhow::Result<()> {
        self.token_bridge
            .set_l2_token_bridge(l2_bridge)
            .await
            .context("Failed to set l2 bridge in starknet token bridge")?;
        Ok(())
    }

    pub async fn deposit(&self, token: Address, amount: U256, l2address: U256, fee: U256) -> anyhow::Result<()> {
        self.token_bridge
            .deposit(token, amount, l2address, fee)
            .await
            .context("Failed to bridge funds from l1 to l2")?;
        Ok(())
    }

    pub async fn withdraw(&self, l1_token: Address, amount: U256, l1_recipient: Address) -> anyhow::Result<()> {
        self.token_bridge
            .withdraw(l1_token, amount, l1_recipient)
            .await
            .context("Failed to withdraw from starknet token bridge")?;
        Ok(())
    }

    pub async fn enroll_token_bridge(&self, address: Address, fee: U256) -> anyhow::Result<()> {
        self.manager
            .enroll_token_bridge(address, fee)
            .await
            .context("Failed to enroll token in starknet token bridge")?;
        Ok(())
    }

    pub async fn approve(&self, address: Address, amount: U256) -> anyhow::Result<()> {
        self.erc20
            .approve(address, amount)
            .await
            .context("Failed to approve dai transfer for starknet token bridge")?;
        Ok(())
    }

    pub async fn token_balance(&self, address: Address) -> anyhow::Result<U256> {
        self.erc20.balance_of(address).await.with_context(|| format!("Getting ERC20 balance for {address}"))
    }
}
