use crate::ArgConfig;
use ethereum_instance::EthereumInstance;
use ethers::types::U256;
use hex::encode;
use starknet_accounts::SingleOwnerAccount;
use starknet_api::hash::{pedersen_hash_array, StarkFelt, StarkHash};
use starknet_core::chain_id;
use starknet_core::types::{BlockId, BlockTag, FunctionCall};
use starknet_core::utils::get_selector_from_name;
use starknet_ff::FieldElement;
use starknet_providers::jsonrpc::HttpTransport;
use starknet_providers::JsonRpcClient;
use starknet_providers::Provider;
use starknet_signers::{LocalWallet, SigningKey};
use url::Url;

pub type RpcAccount<'a> = SingleOwnerAccount<&'a JsonRpcClient<HttpTransport>, LocalWallet>;
pub fn build_single_owner_account<'a>(
    rpc: &'a JsonRpcClient<HttpTransport>,
    private_key: &str,
    account_address: &str,
    is_legacy: bool,
) -> RpcAccount<'a> {
    let signer = LocalWallet::from(SigningKey::from_secret_scalar(
        FieldElement::from_hex_be(private_key).unwrap(),
    ));
    let account_address =
        FieldElement::from_hex_be(account_address).expect("Invalid Contract Address");
    let execution_encoding = if is_legacy {
        starknet_accounts::ExecutionEncoding::Legacy
    } else {
        starknet_accounts::ExecutionEncoding::New
    };
    SingleOwnerAccount::new(
        rpc,
        signer,
        account_address,
        chain_id::TESTNET,
        execution_encoding,
    )
}

pub async fn read_erc20_balance(
    rpc: &JsonRpcClient<HttpTransport>,
    contract_address: FieldElement,
    account_address: FieldElement,
) -> Vec<FieldElement> {
    let balance = rpc
        .call(
            FunctionCall {
                contract_address,
                entry_point_selector: get_selector_from_name("balanceOf").unwrap(),
                calldata: vec![account_address],
            },
            BlockId::Tag(BlockTag::Latest),
        )
        .await
        .unwrap();

    balance
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

pub fn get_bridge_init_configs(config: &ArgConfig) -> (FieldElement, StarkHash) {
    let program_hash = FieldElement::from_hex_be(config.sn_os_program_hash.as_str()).unwrap();
    let config_hash = generate_config_hash(
        FieldElement::from_hex_be(&encode(config.config_hash_version.as_str()))
            .expect("error in config_hash_version"),
        FieldElement::from_hex_be(&encode(config.app_chain_id.as_str()))
            .expect("error in app_chain_id"),
        FieldElement::from_hex_be(config.fee_token_address.as_str())
            .expect("error in fee_token_address"),
    );
    (program_hash, config_hash)
}

#[allow(dead_code)]
pub struct Config {
    eth_client: EthereumInstance,
    provider_l2: JsonRpcClient<HttpTransport>,
}

impl Config {
    pub fn provider_l2(&self) -> &JsonRpcClient<HttpTransport> {
        &self.provider_l2
    }

    pub fn eth_client(&self) -> &EthereumInstance {
        &self.eth_client
    }

    /// To deploy the instance of ethereum and starknet and returning the struct.
    pub async fn init(config: &ArgConfig) -> Self {
        let client_instance = EthereumInstance::spawn(
            config.eth_rpc.clone(),
            config.eth_priv_key.clone(),
            config.eth_chain_id,
        );
        let provider_l2 = JsonRpcClient::new(HttpTransport::new(
            Url::parse(&config.rollup_seq_url).expect("Failed to declare provider for app chain"),
        ));

        Self {
            eth_client: client_instance,
            provider_l2,
        }
    }
}
