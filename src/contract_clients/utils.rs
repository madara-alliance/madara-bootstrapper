use ethers::types::U256;
use hex::encode;
use starknet_accounts::SingleOwnerAccount;
use starknet_api::hash::{pedersen_hash_array, StarkFelt, StarkHash};
use starknet_api::transaction::{DeclareTransactionV0V1, TransactionHash};
use starknet_core::chain_id;
use starknet_core::crypto::compute_hash_on_elements;
use starknet_core::types::{BlockId, BlockTag, FunctionCall};
use starknet_core::utils::get_selector_from_name;
use starknet_ff::FieldElement;
use starknet_providers::jsonrpc::HttpTransport;
use starknet_providers::{JsonRpcClient, Provider};
use starknet_signers::{LocalWallet, SigningKey};

use crate::contract_clients::subxt_funcs::appchain::runtime_types::mp_felt::Felt252Wrapper;
use crate::CliArgs;

pub type RpcAccount<'a> = SingleOwnerAccount<&'a JsonRpcClient<HttpTransport>, LocalWallet>;
pub fn build_single_owner_account<'a>(
    rpc: &'a JsonRpcClient<HttpTransport>,
    private_key: &str,
    account_address: &str,
    is_legacy: bool,
) -> RpcAccount<'a> {
    let signer = LocalWallet::from(SigningKey::from_secret_scalar(FieldElement::from_hex_be(private_key).unwrap()));
    let account_address = FieldElement::from_hex_be(account_address).expect("Invalid Contract Address");
    let execution_encoding = if is_legacy {
        starknet_accounts::ExecutionEncoding::Legacy
    } else {
        starknet_accounts::ExecutionEncoding::New
    };
    SingleOwnerAccount::new(rpc, signer, account_address, chain_id::TESTNET, execution_encoding)
}

pub async fn read_erc20_balance(
    rpc: &JsonRpcClient<HttpTransport>,
    contract_address: FieldElement,
    account_address: FieldElement,
) -> Vec<FieldElement> {
    rpc.call(
        FunctionCall {
            contract_address,
            entry_point_selector: get_selector_from_name("balanceOf").unwrap(),
            calldata: vec![account_address],
        },
        BlockId::Tag(BlockTag::Latest),
    )
    .await
    .unwrap()
}

pub fn field_element_to_u256(input: FieldElement) -> U256 {
    U256::from_big_endian(&input.to_bytes_be())
}

pub fn generate_config_hash(
    config_hash_version: FieldElement,
    chain_id: FieldElement,
    fee_token_address: FieldElement,
) -> StarkHash {
    pedersen_hash_array(&[
        StarkFelt::from(config_hash_version),
        StarkFelt::from(chain_id),
        StarkFelt::from(fee_token_address),
    ])
}

pub fn get_bridge_init_configs(config: &CliArgs) -> (FieldElement, StarkHash) {
    let program_hash = FieldElement::from_hex_be(config.sn_os_program_hash.as_str()).unwrap();
    let config_hash = generate_config_hash(
        FieldElement::from_hex_be(&encode(config.config_hash_version.as_str())).expect("error in config_hash_version"),
        FieldElement::from_hex_be(&encode(config.app_chain_id.as_str())).expect("error in app_chain_id"),
        FieldElement::from_hex_be(config.fee_token_address.as_str()).expect("error in fee_token_address"),
    );
    (program_hash, config_hash)
}
