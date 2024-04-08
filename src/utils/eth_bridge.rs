use std::sync::Arc;

use async_trait::async_trait;
use ethers::addressbook::Address;
use ethers::providers::Middleware;
use ethers::types::{Bytes, U256};
use crate::felt::lib::Felt252Wrapper;
use starknet_accounts::Execution;
use starknet_contract::ContractFactory;
use starknet_eth_bridge_client::clients::eth_bridge::StarknetEthBridgeContractClient;
// zaun imports
use starknet_eth_bridge_client::deploy_starknet_eth_bridge_behind_unsafe_proxy;
use starknet_eth_bridge_client::interfaces::eth_bridge::StarknetEthBridgeTrait;
use starknet_ff::FieldElement;
use starknet_proxy_client::proxy_support::ProxySupportTrait;
use crate::bridge_deploy_utils::lib::constants::LEGACY_BRIDGE_PATH;
use crate::bridge_deploy_utils::lib::fixtures::ThreadSafeMadaraClient;
use crate::bridge_deploy_utils::lib::utils::{build_single_owner_account, AccountActions};
use crate::bridge_deploy_utils::lib::Transaction;
use zaun_utils::{LocalWalletSignerMiddleware, StarknetContractClient};

use super::utils::invoke_contract;

#[async_trait]
pub trait BridgeDeployable {
    async fn deploy(client: Arc<LocalWalletSignerMiddleware>) -> Self;
}

pub struct StarknetLegacyEthBridge {
    eth_bridge: StarknetEthBridgeContractClient,
}

#[async_trait]
impl BridgeDeployable for StarknetLegacyEthBridge {
    async fn deploy(client: Arc<LocalWalletSignerMiddleware>) -> Self {
        let eth_bridge = deploy_starknet_eth_bridge_behind_unsafe_proxy(client.clone())
            .await
            .expect("Failed to deploy starknet contract");

        Self { eth_bridge }
    }
}

impl StarknetLegacyEthBridge {
    pub fn address(&self) -> Address {
        self.eth_bridge.address()
    }

    pub fn client(&self) -> Arc<LocalWalletSignerMiddleware> {
        self.eth_bridge.client()
    }

    pub async fn deploy_l2_contracts(madara: &ThreadSafeMadaraClient, private_key: &str, l2_deployer_address: &str) -> FieldElement {
        let rpc = madara.get_starknet_client().await;
        let account = build_single_owner_account(&rpc, private_key, l2_deployer_address, false);

        let (declare_tx, class_hash) = account.declare_legacy_contract(LEGACY_BRIDGE_PATH);

        let mut madara_write_lock = madara.write().await;

        madara_write_lock
            .create_block_with_txs(vec![Transaction::LegacyDeclaration(declare_tx)])
            .await
            .expect("Unable to declare legacy token bridge on l2");

        let contract_factory = ContractFactory::new(class_hash, account.clone());
        let deploy_tx = &contract_factory.deploy(vec![], FieldElement::ZERO, true);

        madara_write_lock
            .create_block_with_txs(vec![Transaction::Execution(Execution::from(deploy_tx))])
            .await
            .expect("Unable to deploy legacy token bridge on l2");
        deploy_tx.deployed_address()
    }

    /// Initialize Starknet Legacy Eth Bridge
    pub async fn initialize(&self, messaging_contract: Address) {
        let empty_bytes = [0u8; 32];

        let messaging_bytes = messaging_contract.as_bytes();

        let mut padded_messaging_bytes = Vec::with_capacity(32);
        padded_messaging_bytes.extend(vec![0u8; 32 - messaging_bytes.len()]);
        padded_messaging_bytes.extend_from_slice(messaging_bytes);

        let mut calldata = Vec::new();
        calldata.extend(empty_bytes);
        calldata.extend(empty_bytes);
        calldata.extend(padded_messaging_bytes);

        self.eth_bridge.initialize(Bytes::from(calldata)).await.expect("Failed to initialize eth bridge");
    }

    /// Sets up the Eth bridge with the specified data
    pub async fn setup_l1_bridge(&self, max_total_balance: &str, max_deposit: &str, l2_bridge: FieldElement) {
        self.eth_bridge.set_max_total_balance(U256::from_dec_str(max_total_balance).unwrap()).await.unwrap();
        self.eth_bridge.set_max_deposit(U256::from_dec_str(max_deposit).unwrap()).await.unwrap();
        self.eth_bridge.set_l2_token_bridge(U256::from(Felt252Wrapper(l2_bridge))).await.unwrap();
    }

    pub async fn setup_l2_bridge(
        &self,
        madara: &ThreadSafeMadaraClient,
        l2_bridge_address: FieldElement,
        erc20_address: FieldElement,
        priv_key: &str,
        l2_deployer_address: &str
    ) {
        invoke_contract(
            madara,
            l2_bridge_address,
            "initialize",
            vec![
                FieldElement::from_dec_str("1").unwrap(),
                FieldElement::from_hex_be(l2_deployer_address).unwrap(),
            ],
            priv_key,
            l2_deployer_address
        )
        .await;

        invoke_contract(madara, l2_bridge_address, "set_l2_token", vec![erc20_address], priv_key, l2_deployer_address).await;
    
        invoke_contract(
            madara,
            l2_bridge_address,
            "set_l1_bridge",
            vec![FieldElement::from_byte_slice_be(self.eth_bridge.address().as_bytes()).unwrap()],
            priv_key,
            l2_deployer_address
        )
        .await;
    }

    pub async fn set_max_total_balance(&self, amount: U256) {
        self.eth_bridge
            .set_max_total_balance(amount)
            .await
            .expect("Failed to set max total balance value in Eth bridge");
    }

    pub async fn set_max_deposit(&self, amount: U256) {
        self.eth_bridge.set_max_deposit(amount).await.expect("Failed to set max deposit value in eth bridge");
    }

    pub async fn set_l2_token_bridge(&self, l2_bridge: U256) {
        self.eth_bridge.set_l2_token_bridge(l2_bridge).await.expect("Failed to set l2 bridge in eth bridge");
    }

    pub async fn deposit(&self, amount: U256, l2_address: U256, fee: U256) {
        self.eth_bridge.deposit(amount, l2_address, fee).await.expect("Failed to deposit in eth bridge");
    }

    pub async fn withdraw(&self, amount: U256, l1_recipient: Address) {
        self.eth_bridge.withdraw(amount, l1_recipient).await.expect("Failed to withdraw from eth bridge");
    }

    pub async fn eth_balance(&self, l1_recipient: Address) -> U256 {
        let provider = self.eth_bridge.client().provider().clone();

        provider.get_balance(l1_recipient, None).await.unwrap()
    }
}
