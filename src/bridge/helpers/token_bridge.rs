use std::sync::Arc;
use async_trait::async_trait;
use ethers::addressbook::Address;
use ethers::core::rand;
use ethers::prelude::U256;
use ethers::types::Bytes;
use rand::Rng;
use starkgate_manager_client::clients::starkgate_manager::StarkgateManagerContractClient;
use starkgate_manager_client::deploy_starkgate_manager_behind_unsafe_proxy;
use starkgate_manager_client::interfaces::manager::StarkgateManagerTrait;
use starkgate_registry_client::clients::starkgate_registry::StarkgateRegistryContractClient;
use starkgate_registry_client::deploy_starkgate_registry_behind_unsafe_proxy;
use starknet_accounts::{Account};
use starknet_core::utils::get_selector_from_name;
use starknet_erc20_client::clients::erc20::ERC20ContractClient;
use starknet_erc20_client::deploy_dai_erc20_behind_unsafe_proxy;
use starknet_erc20_client::interfaces::erc20::ERC20TokenTrait;
use starknet_ff::FieldElement;
use starknet_providers::jsonrpc::HttpTransport;
use starknet_providers::JsonRpcClient;
use starknet_proxy_client::proxy_support::ProxySupportTrait;
use crate::utils::constants::{
    ERC20_CASM_PATH, ERC20_SIERRA_PATH, TOKEN_BRIDGE_CASM_PATH, TOKEN_BRIDGE_SIERRA_PATH
};
use starknet_token_bridge_client::clients::token_bridge::StarknetTokenBridgeContractClient;
use starknet_token_bridge_client::deploy_starknet_token_bridge_behind_unsafe_proxy;
use starknet_token_bridge_client::interfaces::token_bridge::StarknetTokenBridgeTrait;
use zaun_utils::{LocalWalletSignerMiddleware, StarknetContractClient};
use crate::bridge::helpers::account_actions::{AccountActions, get_contract_address_from_deploy_tx};
use crate::bridge::helpers::deploy_utils::build_single_owner_account;
use crate::bridge::helpers::eth_bridge::BridgeDeployable;
use crate::felt::lib::Felt252Wrapper;
use crate::utils::utils::{invoke_contract, pad_bytes, wait_for_transaction};

pub struct StarknetTokenBridge {
    manager: StarkgateManagerContractClient,
    registry: StarkgateRegistryContractClient,
    token_bridge: StarknetTokenBridgeContractClient,
    erc20: ERC20ContractClient,
}

#[async_trait]
impl BridgeDeployable for StarknetTokenBridge {
    async fn deploy(client: Arc<LocalWalletSignerMiddleware>) -> Self {
        let manager = deploy_starkgate_manager_behind_unsafe_proxy(client.clone())
            .await
            .expect("Failed to deploy starkgate manager contract");
        let registry = deploy_starkgate_registry_behind_unsafe_proxy(client.clone())
            .await
            .expect("Failed to deploy starkgate registry");
        let token_bridge = deploy_starknet_token_bridge_behind_unsafe_proxy(client.clone())
            .await
            .expect("Failed to deploy starknet contract");
        let erc20 =
            deploy_dai_erc20_behind_unsafe_proxy(client.clone()).await.expect("Failed to deploy dai erc20 contract");

        Self { manager, registry, token_bridge, erc20 }
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

    pub async fn deploy_l2_contracts(rpc_provider_l2: &JsonRpcClient<HttpTransport>, priv_key: &str, l2_deployer_address: &str) -> FieldElement {
        let account = build_single_owner_account(&rpc_provider_l2, priv_key, l2_deployer_address, false);
        // ! not needed already declared
        let (class_hash_erc20, contract_artifact_erc20) = account.declare_contract_params_sierra(ERC20_SIERRA_PATH, ERC20_CASM_PATH);
        let (class_hash_bridge, contract_artifact_bridge) = account.declare_contract_params_sierra(TOKEN_BRIDGE_SIERRA_PATH, TOKEN_BRIDGE_CASM_PATH);

        let flattened_class_erc20 = contract_artifact_erc20.flatten().unwrap();
        let flattened_class_bridge = contract_artifact_bridge.flatten().unwrap();

        let declare_txn = account.declare(Arc::new(flattened_class_bridge), class_hash_bridge).send().await.expect("L2 Bridge initiation failed");
        wait_for_transaction(rpc_provider_l2, declare_txn.transaction_hash).await.unwrap();
        // for individual test :
        // let declare_txn_2 = account.declare(Arc::new(flattened_class_erc20), class_hash_erc20).send().await.expect("L2 Bridge initiation failed");
        // wait_for_transaction(rpc_provider_l2, declare_txn_2.transaction_hash).await.unwrap();

        let mut rng = rand::thread_rng();
        let random: u32 = rng.gen();

        let deploy_tx = account.invoke_contract(
            FieldElement::from_hex_be("0x1").unwrap(),
            "deploy_contract",
            vec![
                FieldElement::from_hex_be("0x0358663e6ed9d37efd33d4661e20b2bad143e0f92076b0c91fe65f31ccf55046")
                    .unwrap(), // class_hash
                FieldElement::from_dec_str(&random.to_string()).unwrap(), // contract_address_salt
                FieldElement::ONE,                                        // constructor_calldata_len
                FieldElement::ZERO,                                       // constructor_calldata (upgrade_delay)
            ],
            None,
        ).send().await.expect("");

        wait_for_transaction(rpc_provider_l2, deploy_tx.transaction_hash).await.unwrap();

        get_contract_address_from_deploy_tx(&rpc_provider_l2, &deploy_tx).await.unwrap()
    }

    /// Initialize Starknet Token Bridge.
    pub async fn initialize(&self, messaging_contract: Address) {
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

        self.manager.initialize(Bytes::from(manager_calldata)).await.expect("Failed to initialize starkgate manager");
        self.registry
            .initialize(Bytes::from(registry_calldata))
            .await
            .expect("Failed to initialize starkgate registry");
        self.token_bridge
            .initialize(Bytes::from(bridge_calldata))
            .await
            .expect("Failed to initialize starknet token bridge");
    }

    /// Sets up the Token bridge with the specified data
    pub async fn setup_l1_bridge(&self, governor: Address, l2_bridge: FieldElement, fee: U256) {
        self.token_bridge.register_app_role_admin(governor).await.unwrap();
        self.token_bridge.register_app_governor(governor).await.unwrap();
        self.token_bridge.set_l2_token_bridge(U256::from(Felt252Wrapper(l2_bridge))).await.unwrap();
        self.manager.enroll_token_bridge(self.address(), fee).await.unwrap();
    }

    pub async fn setup_l2_bridge(&self, rpc_provider_l2: &JsonRpcClient<HttpTransport>, l2_bridge: FieldElement, priv_key: &str, l2_address: &str) {
        let tx = invoke_contract(
            rpc_provider_l2,
            FieldElement::from_hex_be("0x1").unwrap(),
            "__execute__",
            vec![
                l2_bridge,                                                  // contract_address
                get_selector_from_name("register_app_role_admin").unwrap(), // selector
                FieldElement::ONE,                                          // calldata_len
                FieldElement::from_hex_be(l2_address).unwrap(),             // admin_address
            ],
            priv_key,
            l2_address
        )
        .await;

        wait_for_transaction(rpc_provider_l2, tx.transaction_hash).await.unwrap();
        log::debug!("setup_l2_bridge : register_app_role_admin //");

        let tx = invoke_contract(
            rpc_provider_l2,
            l2_bridge,
            "register_app_governor",
            vec![FieldElement::from_hex_be(l2_address).unwrap()],
            priv_key,
            l2_address
        )
        .await;

        wait_for_transaction(rpc_provider_l2, tx.transaction_hash).await.unwrap();
        log::debug!("setup_l2_bridge : register_app_governor //");

        let tx = invoke_contract(
            rpc_provider_l2,
            l2_bridge,
            "set_l2_token_governance",
            vec![FieldElement::from_hex_be(l2_address).unwrap()],
            priv_key,
            l2_address
        )
        .await;

        wait_for_transaction(rpc_provider_l2, tx.transaction_hash).await.unwrap();
        log::debug!("setup_l2_bridge : set_l2_token_governance //");

        let tx = invoke_contract(
            rpc_provider_l2,
            l2_bridge,
            "set_erc20_class_hash",
            vec![
                FieldElement::from_hex_be("0x008b150cfa4db35ed9d685d79f6daa590ff2bb10c295cd656fcbf176c4bd8365")
                    .unwrap(), // class hash
            ],
            priv_key,
            l2_address
        )
        .await;

        wait_for_transaction(rpc_provider_l2, tx.transaction_hash).await.unwrap();
        log::debug!("setup_l2_bridge : set_erc20_class_hash //");

        let tx = invoke_contract(
            rpc_provider_l2,
            l2_bridge,
            "set_l1_bridge",
            vec![FieldElement::from_byte_slice_be(self.token_bridge.address().as_bytes()).unwrap()],
            priv_key,
            l2_address
        )
        .await;
        wait_for_transaction(rpc_provider_l2, tx.transaction_hash).await.unwrap();
        log::debug!("setup_l2_bridge : set_l1_bridge //");
    }

    pub async fn register_app_role_admin(&self, address: Address) {
        self.token_bridge
            .register_app_role_admin(address)
            .await
            .expect("Failed to register app role admin in starknet token bridge");
    }

    pub async fn register_app_governor(&self, address: Address) {
        self.token_bridge
            .register_app_governor(address)
            .await
            .expect("Failed to register app governor in starknet token bridge");
    }

    pub async fn set_l2_token_bridge(&self, l2_bridge: U256) {
        self.token_bridge
            .set_l2_token_bridge(l2_bridge)
            .await
            .expect("Failed to set l2 bridge in starknet token bridge");
    }

    pub async fn deposit(&self, token: Address, amount: U256, l2address: U256, fee: U256) {
        self.token_bridge.deposit(token, amount, l2address, fee).await.expect("Failed to bridge funds from l1 to l2");
    }

    pub async fn withdraw(&self, l1_token: Address, amount: U256, l1_recipient: Address) {
        self.token_bridge
            .withdraw(l1_token, amount, l1_recipient)
            .await
            .expect("Failed to withdraw from starknet token bridge");
    }

    pub async fn enroll_token_bridge(&self, address: Address, fee: U256) {
        self.manager.enroll_token_bridge(address, fee).await.expect("Failed to enroll token in starknet token bridge");
    }

    pub async fn approve(&self, address: Address, amount: U256) {
        self.erc20
            .approve(address, amount)
            .await
            .expect("Failed to approve dai transfer for starknet token bridge");
    }

    pub async fn token_balance(&self, address: Address) -> U256 {
        self.erc20.balance_of(address).await.unwrap()
    }
}
