use std::sync::Arc;

use ethers::abi::AbiEncode;
use ethers::types::{Address, Bytes, I256, U256};
use starknet_api::hash::{StarkFelt, StarkHash};
use starknet_core_contract_client::clients::StarknetValidityContractClient;
use starknet_core_contract_client::deploy_starknet_validity_behind_safe_proxy;
use starknet_core_contract_client::interfaces::{OperatorTrait, StarknetGovernanceTrait};
use starknet_ff::FieldElement;
use starknet_proxy_client::proxy_support::{
    CoreContractInitData, CoreContractState, ProxyInitializeData, ProxySupportTrait,
};
use zaun_utils::{LocalWalletSignerMiddleware, StarknetContractClient};

use crate::contract_clients::config::Config;
use crate::utils::convert_felt_to_u256;

pub struct StarknetValidityContract {
    core_contract_client: StarknetValidityContractClient,
}

impl StarknetValidityContract {
    pub fn address(&self) -> Address {
        self.core_contract_client.address()
    }

    pub fn implementation_address(&self) -> Address {
        log::debug!(
            "🎡 self.core_contract_client.implementation_address() : {:?}",
            self.core_contract_client.implementation_address()
        );
        self.core_contract_client.implementation_address()
    }

    pub fn client(&self) -> Arc<LocalWalletSignerMiddleware> {
        self.core_contract_client.client()
    }

    pub async fn deploy(config: &Config) -> Self {
        let client = deploy_starknet_validity_behind_safe_proxy(config.eth_client().signer().clone())
            .await
            .expect("Failed to deploy the starknet contact");

        Self { core_contract_client: client }
    }

    /// Initialize Starknet core contract with the specified data.
    /// IMP : only need to be called when using unsafe proxy
    pub async fn initialize_with(&self, init_data: CoreContractInitData) {
        let data = ProxyInitializeData::<0> { sub_contract_addresses: [], eic_address: Default::default(), init_data };

        log::debug!("ℹ️  initialize_with : data : {:?}", data);

        self.core_contract_client.initialize_with(data).await.expect("Failed to initialize");

        self.core_contract_client
            .register_operator(self.core_contract_client.client().address())
            .await
            .expect("Failed to register operator");
    }

    /// Add implementation Starknet core contract with the specified data.
    #[allow(clippy::too_many_arguments)]
    pub async fn add_implementation_core_contract(
        &self,
        block_number: StarkFelt,
        state_root: StarkFelt,
        program_hash: FieldElement,
        config_hash: StarkHash,
        implementation_address: Address,
        verifier_address: Address,
        finalized: bool,
    ) {
        let program_hash = StarkFelt(program_hash.to_bytes_be());

        let init_data =
            Self::get_init_data_core_contract(block_number, state_root, program_hash, config_hash, verifier_address);
        let final_bytes = Self::get_calldata_bytes(init_data.clone());

        log::debug!("ℹ️  add_implementation : data : {:?} : {:?}", init_data, final_bytes.clone());

        // https://sepolia.etherscan.io/tx/0x9ac02beb912e5c6226828110380d727a6fd7e4748cbded2198cdf62ea78dab62
        // let bytes_etherscan =
        // Bytes::from_str("
        // 0x0000000000000000000000000000000000000000000000000000000000000000001f0f5ba973c4d890fc2f1c18c33b87f487839fb44b9019e08d37d2ab943055000000000000000000000000f294781d719d2f4169ce54469c28908e6fa752c1044b9bae3e41deeeeeefe4dc6260a368e88458baab232bd5c89418063e4550c20000000000000000000000000000000000000000000000000000000000000000ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0000000000000000000000000000000000000000000000000000000000000000"
        // ).unwrap();

        self.core_contract_client
            .add_implementation(final_bytes, implementation_address, finalized)
            .await
            .expect("Failed to call add implementation");

        log::debug!("ℹ️  add_implementation : done");
    }

    /// Add implementation Starknet core contract with the specified data.
    #[allow(clippy::too_many_arguments)]
    pub async fn upgrade_to_core_contract(
        &self,
        block_number: StarkFelt,
        state_root: StarkFelt,
        program_hash: FieldElement,
        config_hash: StarkHash,
        implementation_address: Address,
        verifier_address: Address,
        finalized: bool,
    ) {
        let program_hash = StarkFelt(program_hash.to_bytes_be());

        let init_data =
            Self::get_init_data_core_contract(block_number, state_root, program_hash, config_hash, verifier_address);
        let final_bytes = Self::get_calldata_bytes(init_data.clone());

        log::debug!("ℹ️  upgrade_to : data : {:?} : {:?}", init_data, final_bytes.clone());

        // https://sepolia.etherscan.io/tx/0x9ac02beb912e5c6226828110380d727a6fd7e4748cbded2198cdf62ea78dab62
        // let bytes_etherscan =
        // Bytes::from_str("
        // 0x0000000000000000000000000000000000000000000000000000000000000000001f0f5ba973c4d890fc2f1c18c33b87f487839fb44b9019e08d37d2ab943055000000000000000000000000f294781d719d2f4169ce54469c28908e6fa752c1044b9bae3e41deeeeeefe4dc6260a368e88458baab232bd5c89418063e4550c20000000000000000000000000000000000000000000000000000000000000000ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0000000000000000000000000000000000000000000000000000000000000000"
        // ).unwrap();

        self.core_contract_client
            .upgrade_to(final_bytes, implementation_address, finalized)
            .await
            .expect("Failed to call upgrade to");

        log::debug!("ℹ️  upgrade_to : done");
    }

    /// For registering the operator for Starknet Core Contract
    pub async fn register_operator_core_contract(&self, operator_address: Address) {
        self.core_contract_client.register_operator(operator_address).await.expect("Failed to register operator");
        log::debug!("ℹ️  register_operator : done");
    }

    /// For nominating the governor for Starknet Core Contract
    pub async fn nominate_governor_core_contract(&self, l1_governor_address: Address) {
        self.core_contract_client
            .starknet_nominate_new_governor(l1_governor_address)
            .await
            .expect("Failed to nominate governor");
        log::debug!("ℹ️  register_operator : done");
    }

    /// For nominating the governor for Starknet Core Contract Proxy
    pub async fn nominate_governor_core_contract_proxy(&self, l1_governor_address: Address) {
        self.core_contract_client
            .proxy_nominate_new_governor(l1_governor_address)
            .await
            .expect("Failed to register operator");
        log::debug!("ℹ️  proxy_nominate_new_governor : done");
    }

    /// Initialize Starknet core contract with the specified program and config hashes. The rest of
    /// parameters will be left default.
    /// IMP : only need to be called when using unsafe proxy
    pub async fn initialize(&self, program_hash: StarkFelt, config_hash: StarkFelt) {
        self.initialize_with(CoreContractInitData {
            program_hash: convert_felt_to_u256(program_hash),
            config_hash: convert_felt_to_u256(config_hash),
            ..Default::default()
        })
        .await;
    }

    /// Initialize Starknet core contract with the specified block number and state root hash.
    /// IMP : only need to be called when using unsafe proxy
    pub async fn initialize_core_contract(
        &self,
        block_number: StarkFelt,
        state_root: StarkFelt,
        program_hash: FieldElement,
        config_hash: StarkHash,
        verifer_address: Address,
    ) {
        let program_hash = StarkFelt(program_hash.to_bytes_be());

        let init_data =
            Self::get_init_data_core_contract(block_number, state_root, program_hash, config_hash, verifer_address);

        self.initialize_with(init_data).await;
    }

    fn get_init_data_core_contract(
        block_number: StarkFelt,
        state_root: StarkFelt,
        program_hash: StarkFelt,
        config_hash: StarkHash,
        verifier_address: Address,
    ) -> CoreContractInitData {
        CoreContractInitData {
            program_hash: convert_felt_to_u256(program_hash), // zero program hash would be deemed invalid
            verifier_address,
            config_hash: convert_felt_to_u256(config_hash),
            // TODO :
            // Figure out the exact params for production env
            initial_state: CoreContractState {
                block_number: I256::from_raw(convert_felt_to_u256(block_number)),
                state_root: convert_felt_to_u256(state_root),
                // TODO :
                // Remove hardcoded values.
                block_hash: U256::zero(),
            },
        }
    }

    fn get_calldata_bytes(calldata: CoreContractInitData) -> Bytes {
        let mut bytes_final = Address::zero().encode();
        let bytes: Vec<u8> = <CoreContractInitData as Into<Vec<u8>>>::into(calldata.clone());
        for x in bytes {
            bytes_final.push(x);
        }

        Bytes::from(bytes_final)
    }
}
