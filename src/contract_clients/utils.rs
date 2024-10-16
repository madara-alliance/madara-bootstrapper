use std::sync::Arc;

use ethers::types::U256;
use hex::encode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use starknet::accounts::{
    Account, AccountFactory, ConnectedAccount, ExecutionEncoding, OpenZeppelinAccountFactory, SingleOwnerAccount,
};
use starknet::core::types::contract::legacy::LegacyContractClass;
use starknet::core::types::{BlockId, BlockTag, DeclareTransactionResult, Felt, FunctionCall};
use starknet::core::utils::get_selector_from_name;
use starknet::providers::jsonrpc::{HttpTransport, JsonRpcClient};
use starknet::providers::Provider;
use starknet::signers::{LocalWallet, SigningKey};
use starknet_core::types::contract::{CompiledClass, SierraClass};
use starknet_core::types::BlockTag::Pending;
use starknet_types_core::hash::{Pedersen, StarkHash};

use crate::contract_clients::legacy_class::CompressedLegacyContractClass;
use crate::contract_clients::utils::DeclarationInput::{DeclarationInputs, LegacyDeclarationInputs};
use crate::helpers::account_actions::{get_contract_address_from_deploy_tx, AccountActions};
use crate::utils::{invoke_contract, save_to_json, wait_for_transaction, JsonValueType};
use crate::CliArgs;

pub type RpcAccount<'a> = SingleOwnerAccount<&'a JsonRpcClient<HttpTransport>, LocalWallet>;
pub async fn build_single_owner_account<'a>(
    rpc: &'a JsonRpcClient<HttpTransport>,
    private_key: &str,
    account_address: &str,
    is_legacy: bool,
) -> RpcAccount<'a> {
    let signer = LocalWallet::from(SigningKey::from_secret_scalar(Felt::from_hex(private_key).unwrap()));
    let account_address = Felt::from_hex(account_address).expect("Invalid Contract Address");
    let execution_encoding = if is_legacy { ExecutionEncoding::Legacy } else { ExecutionEncoding::New };

    let chain_id = rpc.chain_id().await.unwrap();

    // Note: it's a fix for the starknet rs issue, by default, starknet.rs asks for nonce at the latest
    // block which causes the issues hence setting the block id to pending so that we get nonce in
    // right order
    let mut singer_with_pending_id =
        SingleOwnerAccount::new(rpc, signer, account_address, chain_id, execution_encoding);
    singer_with_pending_id.set_block_id(BlockId::Tag(Pending));
    singer_with_pending_id
}

pub async fn read_erc20_balance(
    rpc: &JsonRpcClient<HttpTransport>,
    contract_address: Felt,
    account_address: Felt,
) -> Vec<Felt> {
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

pub fn field_element_to_u256(input: Felt) -> U256 {
    U256::from_big_endian(&input.to_bytes_be())
}

pub fn generate_config_hash(
    config_hash_version: Felt,
    chain_id: Felt,
    fee_token_address: Felt,
    native_fee_token_address: Felt,
) -> Felt {
    Pedersen::hash_array(&[config_hash_version, chain_id, fee_token_address, native_fee_token_address])
}

pub fn get_bridge_init_configs(config: &CliArgs) -> (Felt, Felt) {
    let program_hash = Felt::from_hex(config.sn_os_program_hash.as_str()).unwrap();

    let config_hash = generate_config_hash(
        Felt::from_hex(&encode(config.config_hash_version.as_str())).expect("error in config_hash_version"),
        Felt::from_hex(&encode(config.app_chain_id.as_str())).expect("error in app_chain_id"),
        Felt::from_hex(config.fee_token_address.as_str()).expect("error in fee_token_address"),
        Felt::from_hex(config.native_fee_token_address.as_str()).expect("error in fee_token_address"),
    );
    (program_hash, config_hash)
}

/// Broadcasted declare contract transaction v0.
#[derive(Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BroadcastedDeclareTransactionV0 {
    /// The address of the account contract sending the declaration transaction
    pub sender_address: Felt,
    /// The maximal fee that can be charged for including the transaction
    pub max_fee: Felt,
    /// Signature
    pub signature: Vec<Felt>,
    /// The class to be declared
    pub contract_class: Arc<CompressedLegacyContractClass>,
    /// If set to `true`, uses a query-only transaction version that's invalid for execution
    pub is_query: bool,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct RpcResult<T> {
    jsonrpc: String,
    result: T,
    id: u64,
}

pub const TEMP_ACCOUNT_PRIV_KEY: &str = "0xbeef";

pub(crate) enum DeclarationInput<'a> {
    // inputs : sierra_path, casm_path
    DeclarationInputs(String, String, RpcAccount<'a>),
    // input : artifact_path
    LegacyDeclarationInputs(String, String),
}

#[allow(private_interfaces)]
pub async fn declare_contract(input: DeclarationInput<'_>) -> Felt {
    match input {
        DeclarationInputs(sierra_path, casm_path, account) => {
            let contract_artifact: SierraClass =
                serde_json::from_reader(std::fs::File::open(sierra_path).unwrap()).unwrap();

            let contract_artifact_casm: CompiledClass =
                serde_json::from_reader(std::fs::File::open(casm_path).unwrap()).unwrap();
            let class_hash = contract_artifact_casm.class_hash().unwrap();
            let sierra_class_hash = contract_artifact.class_hash().unwrap();

            let flattened_class = contract_artifact.flatten().unwrap();

            account
                .declare_v3(Arc::new(flattened_class), class_hash)
                .gas(0)
                .send()
                .await
                .expect("Error in declaring the contract using Cairo 1 declaration using the provided account");
            sierra_class_hash
        }
        LegacyDeclarationInputs(artifact_path, url) => {
            let contract_abi_artifact_temp: LegacyContractClass = serde_json::from_reader(
                std::fs::File::open(env!("CARGO_MANIFEST_DIR").to_owned() + "/" + &artifact_path).unwrap(),
            )
            .unwrap();

            let contract_abi_artifact: CompressedLegacyContractClass = contract_abi_artifact_temp
                .clone()
                .compress()
                .expect("Error : Failed to compress the contract class")
                .into();

            let params: BroadcastedDeclareTransactionV0 = BroadcastedDeclareTransactionV0 {
                sender_address: Felt::from_hex("0x1").unwrap(),
                max_fee: Felt::ZERO,
                signature: Vec::new(),
                contract_class: Arc::new(contract_abi_artifact),
                is_query: false,
            };

            // TODO: method can be updated based on the madara PR
            let json_body = &json!({
                "jsonrpc": "2.0",
                "method": "madara_addDeclareV0Transaction",
                "params": [params],
                "id": 4
            });

            let req_client = reqwest::Client::new();
            let raw_txn_rpc = req_client.post(url).json(json_body).send().await;
            match raw_txn_rpc {
                Ok(val) => {
                    log::debug!(
                        "🚧 Txn Sent Successfully : {:?}",
                        val.json::<RpcResult<DeclareTransactionResult>>().await.unwrap()
                    );
                }
                Err(err) => {
                    log::debug!("Error : Error sending the transaction using RPC");
                    log::debug!("{:?}", err);
                }
            }

            contract_abi_artifact_temp.class_hash().unwrap()
        }
    }
}

pub(crate) async fn deploy_account_using_priv_key(
    priv_key: String,
    provider: &JsonRpcClient<HttpTransport>,
    oz_account_class_hash: Felt,
) -> Felt {
    let chain_id = provider.chain_id().await.unwrap();

    let signer = LocalWallet::from(SigningKey::from_secret_scalar(Felt::from_hex(&priv_key).unwrap()));
    log::trace!("signer : {:?}", signer);
    let mut oz_account_factory =
        OpenZeppelinAccountFactory::new(oz_account_class_hash, chain_id, signer, provider).await.unwrap();
    oz_account_factory.set_block_id(BlockId::Tag(BlockTag::Pending));

    let deploy_txn = oz_account_factory.deploy_v1(Felt::ZERO).max_fee(Felt::ZERO);
    let account_address = deploy_txn.address();
    log::trace!("OZ Account Deployed : {:?}", account_address);
    save_to_json("account_address", &JsonValueType::StringType(account_address.to_string())).unwrap();

    let sent_txn = deploy_txn.send().await.expect("Error in deploying the OZ account");

    log::trace!("deploy account txn_hash : {:?}", sent_txn.transaction_hash);

    wait_for_transaction(provider, sent_txn.transaction_hash, "deploy_account_using_priv_key").await.unwrap();

    account_address
}

pub(crate) async fn deploy_proxy_contract(
    account: &RpcAccount<'_>,
    account_address: Felt,
    class_hash: Felt,
    salt: Felt,
    deploy_from_zero: Felt,
) -> Felt {
    let txn = account
        .invoke_contract(
            account_address,
            "deploy_contract",
            vec![class_hash, salt, deploy_from_zero, Felt::ONE, Felt::ZERO],
            None,
        )
        .send()
        .await
        .expect("Error deploying the contract proxy.");

    log::debug!("txn in proxy contract is: {:?}", txn);

    wait_for_transaction(account.provider(), txn.transaction_hash, "deploy_proxy_contract : deploy_contract")
        .await
        .unwrap();

    log::trace!("txn hash (proxy deployment) : {:?}", txn.transaction_hash);

    let deployed_address = get_contract_address_from_deploy_tx(account.provider(), &txn).await.unwrap();
    log::trace!("[IMP] Event : {:?}", deployed_address);

    deployed_address
}

pub(crate) async fn init_governance_proxy(account: &'_ RpcAccount<'_>, contract_address: Felt, tag: &str) {
    let txn = invoke_contract(contract_address, "init_governance", vec![], account).await;
    wait_for_transaction(account.provider(), txn.transaction_hash, tag).await.unwrap();
}
