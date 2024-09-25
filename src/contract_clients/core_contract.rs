use std::future::Future;
use std::sync::Arc;

use async_trait::async_trait;
use ethers::abi::AbiEncode;
use ethers::addressbook::Address;
use ethers::prelude::{Bytes, I256, U256};
use starknet_api::hash::{StarkFelt, StarkHash};
use starknet_ff::FieldElement;
use starknet_proxy_client::interfaces::proxy::{CoreContractInitData, CoreContractState};
use zaun_utils::LocalWalletSignerMiddleware;

use crate::contract_clients::config::Config;
use crate::utils::convert_felt_to_u256;

#[async_trait]
pub trait CoreContract {
    fn address(&self) -> Address;

    fn implementation_address(&self) -> Address;

    fn client(&self) -> Arc<LocalWalletSignerMiddleware>;

    async fn initialize_with(&self, init_data: CoreContractInitData) -> anyhow::Result<()>;

    #[allow(clippy::too_many_arguments)]
    async fn add_implementation_core_contract(
        &self,
        block_number: StarkFelt,
        state_root: StarkFelt,
        program_hash: FieldElement,
        config_hash: StarkHash,
        implementation_address: Address,
        verifier_address: Address,
        finalized: bool,
    ) -> anyhow::Result<()>;

    #[allow(clippy::too_many_arguments)]
    async fn upgrade_to_core_contract(
        &self,
        block_number: StarkFelt,
        state_root: StarkFelt,
        program_hash: FieldElement,
        config_hash: StarkHash,
        implementation_address: Address,
        verifier_address: Address,
        finalized: bool,
    ) -> anyhow::Result<()>;

    async fn register_operator_core_contract(&self, operator_address: Address) -> anyhow::Result<()>;

    async fn nominate_governor_core_contract(&self, l1_governor_address: Address) -> anyhow::Result<()>;

    async fn nominate_governor_core_contract_proxy(&self, l1_governor_address: Address) -> anyhow::Result<()>;

    async fn initialize(&self, program_hash: StarkFelt, config_hash: StarkFelt) -> anyhow::Result<()>;

    async fn initialize_core_contract(
        &self,
        block_number: StarkFelt,
        state_root: StarkFelt,
        program_hash: FieldElement,
        config_hash: StarkHash,
        verifer_address: Address,
    ) -> anyhow::Result<()>;
}

pub trait CoreContractDeploy<T> {
    fn deploy(config: &Config) -> impl Future<Output = anyhow::Result<T>> + Send;
}

pub fn get_init_data_core_contract(
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

pub fn get_calldata_bytes(calldata: CoreContractInitData) -> Bytes {
    let mut bytes_final = Address::zero().encode();
    let bytes: Vec<u8> = <CoreContractInitData as Into<Vec<u8>>>::into(calldata.clone());
    for x in bytes {
        bytes_final.push(x);
    }

    Bytes::from(bytes_final)
}
