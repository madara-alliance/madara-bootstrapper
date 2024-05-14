use std::sync::Arc;

use ethers::types::{Address, I256};
use starknet_api::hash::{StarkFelt, StarkHash};
use starknet_core_contract_client::clients::StarknetSovereignContractClient;
use starknet_core_contract_client::deploy_starknet_sovereign_behind_unsafe_proxy;
use starknet_core_contract_client::interfaces::OperatorTrait;
use starknet_ff::FieldElement;
use starknet_proxy_client::proxy_support::{
    CoreContractInitData, CoreContractState, ProxyInitializeData, ProxySupportTrait,
};
use zaun_utils::{LocalWalletSignerMiddleware, StarknetContractClient};

use crate::contract_clients::config::Config;
use crate::utils::convert_felt_to_u256;

pub struct StarknetSovereignContract {
    core_contract_client: StarknetSovereignContractClient,
}

impl StarknetSovereignContract {
    pub fn address(&self) -> Address {
        self.core_contract_client.address()
    }

    pub fn client(&self) -> Arc<LocalWalletSignerMiddleware> {
        self.core_contract_client.client()
    }

    pub async fn deploy(config: &Config) -> Self {
        let client = deploy_starknet_sovereign_behind_unsafe_proxy(config.eth_client().signer().clone())
            .await
            .expect("Failed to deploy the starknet contact");

        Self { core_contract_client: client }
    }

    /// Initialize Starknet core contract with the specified data.
    pub async fn initialize_with(&self, init_data: CoreContractInitData) {
        let data = ProxyInitializeData::<0> { sub_contract_addresses: [], eic_address: Default::default(), init_data };

        self.core_contract_client.initialize_with(data).await.expect("Failed to initialize");

        self.core_contract_client
            .register_operator(self.core_contract_client.client().address())
            .await
            .expect("Failed to register operator");
    }

    /// Initialize Starknet core contract with the specified program and config hashes. The rest of
    /// parameters will be left default.
    pub async fn initialize(&self, program_hash: StarkFelt, config_hash: StarkFelt) {
        self.initialize_with(CoreContractInitData {
            program_hash: convert_felt_to_u256(program_hash),
            config_hash: convert_felt_to_u256(config_hash),
            ..Default::default()
        })
        .await;
    }

    /// Initialize Starknet core contract with the specified block number and state root hash.
    pub async fn initialize_core_contract(
        &self,
        block_number: StarkFelt,
        state_root: StarkFelt,
        program_hash: FieldElement,
        config_hash: StarkHash,
    ) {
        let program_hash = StarkFelt {
            0: program_hash.to_bytes_be(),
        };

        let init_data = CoreContractInitData {
            program_hash: convert_felt_to_u256(program_hash), // zero program hash would be deemed invalid
            config_hash: convert_felt_to_u256(config_hash),
            // TODO :
            // Figure out the exact params for production env
            initial_state: CoreContractState {
                block_number: I256::from_raw(convert_felt_to_u256(block_number)),
                state_root: convert_felt_to_u256(state_root),
                ..Default::default()
            },
            ..Default::default()
        };

        self.initialize_with(init_data).await;
    }
}
