use std::future::Future;
use std::str::FromStr;
use std::sync::Arc;

use async_trait::async_trait;
use ethers::abi::AbiEncode;
use ethers::addressbook::Address;
use ethers::prelude::{Bytes, I256, U256};
use starknet::core::types::Felt;
use starknet_proxy_client::interfaces::proxy::{CoreContractInitData, CoreContractState};
use zaun_utils::LocalWalletSignerMiddleware;

use crate::contract_clients::config::Config;
use crate::utils::convert_felt_to_u256;

#[async_trait]
pub trait CoreContract {
    fn address(&self) -> Address;

    fn implementation_address(&self) -> Address;

    fn client(&self) -> Arc<LocalWalletSignerMiddleware>;

    async fn initialize_with(&self, init_data: CoreContractInitData);

    #[allow(clippy::too_many_arguments)]
    async fn add_implementation_core_contract(
        &self,
        block_number: Felt,
        state_root: Felt,
        program_hash: Felt,
        config_hash: Felt,
        implementation_address: Address,
        verifier_address: Address,
        finalized: bool,
    );

    #[allow(clippy::too_many_arguments)]
    async fn upgrade_to_core_contract(
        &self,
        block_number: Felt,
        state_root: Felt,
        program_hash: Felt,
        config_hash: Felt,
        implementation_address: Address,
        verifier_address: Address,
        finalized: bool,
    );

    async fn register_operator_core_contract(&self, operator_address: Address);

    async fn nominate_governor_core_contract(&self, l1_governor_address: Address);

    async fn nominate_governor_core_contract_proxy(&self, l1_governor_address: Address);

    async fn initialize(&self, program_hash: Felt, config_hash: Felt);

    async fn initialize_core_contract(
        &self,
        block_number: Felt,
        state_root: Felt,
        program_hash: Felt,
        config_hash: Felt,
        verifer_address: Address,
    );
}

pub trait CoreContractDeploy<T> {
    fn deploy(config: &Config) -> impl Future<Output = T> + Send;
}

pub fn get_init_data_core_contract(
    _block_number: Felt,
    _state_root: Felt,
    _program_hash: Felt,
    config_hash: Felt,
    _verifier_address: Address,
) -> CoreContractInitData {
    CoreContractInitData {
        program_hash: U256::from_str_radix("1e324682835e60c4779a683b32713504aed894fd73842f7d05b18e7bd29cd70", 16)
            .unwrap(), // zero program hash would be deemed invalid
        aggregate_program_hash: U256::zero(),
        verifier_address: Address::from_str("0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512").unwrap(),
        config_hash: convert_felt_to_u256(config_hash),
        // TODO :
        // Figure out the exact params for production env
        initial_state: CoreContractState {
            block_number: I256::from(11),
            state_root: U256::from_str_radix("5f41b32fadc9d1bd9d2bf2eb8771e1c64b2ad1b6f3334bf6be0b38c408e4746", 16)
                .unwrap(),
            // TODO :
            // Remove hardcoded values.
            block_hash: U256::from_str_radix("6aeb708c2a47182fd921db56803102eabbd2940c75e0c306012fff144b02186", 16)
                .unwrap(),
        },
    }
}

pub fn get_calldata_bytes(calldata: CoreContractInitData) -> Bytes {
    let mut bytes_final = Address::zero().encode();
    let bytes: Vec<u8> = <CoreContractInitData as Into<Vec<u8>>>::into(calldata.clone());
    for x in bytes {
        bytes_final.push(x);
    }

    Bytes::from(bytes_final)
}
