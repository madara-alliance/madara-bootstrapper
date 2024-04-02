use ethers::types::U256;
use starknet_api::hash::StarkFelt;

pub mod arg_config;
pub mod deploy_utils;
pub mod deploy_eth_bridge;
pub mod eth_bridge;
pub mod utils;

pub fn convert_felt_to_u256(felt: StarkFelt) -> U256 {
    U256::from_big_endian(felt.bytes())
}