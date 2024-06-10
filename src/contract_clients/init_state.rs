use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use blockifier::execution::contract_class::ContractClassV0Inner;
use indexmap::IndexMap;
use parity_scale_codec::Encode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use starknet_accounts::{Account, AccountFactory, ConnectedAccount, OpenZeppelinAccountFactory};
use starknet_api::core::{ClassHash, ContractAddress, Nonce, PatriciaKey};
use starknet_api::deprecated_contract_class::{EntryPoint, EntryPointType};
use starknet_api::hash::{StarkFelt, StarkHash};
use starknet_api::transaction::{DeclareTransactionV0V1, Fee, TransactionHash, TransactionSignature};
use starknet_core::types::contract::legacy::LegacyContractClass;
use starknet_core::types::{BlockId, BlockTag};
use starknet_ff::FieldElement;
use starknet_providers::jsonrpc::HttpTransport;
use starknet_providers::{JsonRpcClient, Provider};
use starknet_signers::{LocalWallet, SigningKey};
use tokio::time::sleep;

use crate::bridge::helpers::account_actions::{get_contract_address_from_deploy_tx, AccountActions};
use crate::contract_clients::config::Config;
use crate::contract_clients::utils::{build_single_owner_account, RpcAccount};
use crate::utils::constants::{
    ERC20_CASM_PATH, ERC20_LEGACY_PATH, ERC20_SIERRA_PATH, LEGACY_BRIDGE_PATH, OZ_ACCOUNT_CASM_PATH, OZ_ACCOUNT_PATH,
    OZ_ACCOUNT_SIERRA_PATH, PROXY_LEGACY_PATH, STARKGATE_PROXY_PATH,
};
use crate::utils::{convert_to_hex, invoke_contract, save_to_json, wait_for_transaction, JsonValueType};
use crate::CliArgs;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct CustomDeclareV0Transaction {
    pub declare_transaction: DeclareTransactionV0V1,
    pub program_vec: Vec<u8>,
    pub entrypoints: IndexMap<EntryPointType, Vec<EntryPoint>>,
    pub abi_length: usize,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct RpcResult<T> {
    jsonrpc: String,
    result: T,
    id: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeclareV0Result {
    pub txn_hash: TransactionHash,
    pub class_hash: ClassHash,
}

pub const TEMP_ACCOUNT_PRIV_KEY: &str = "0xbeef";

pub async fn init_and_deploy_eth_and_account(
    clients: &Config,
    arg_config: &CliArgs,
) -> (
    FieldElement,
    FieldElement,
    FieldElement,
    FieldElement,
    FieldElement,
    FieldElement,
    FieldElement,
    FieldElement,
    FieldElement,
    FieldElement,
) {
    // toggle_fee(true).await.expect("Error in disabling the fee on configured app-chain");
    let legacy_proxy_class_hash = declare_contract_middleware(DeclarationInput::LegacyDeclarationInputs(
        String::from(PROXY_LEGACY_PATH),
        arg_config.rollup_seq_url.clone(),
    ))
    .await;
    log::debug!("Legacy Proxy Class Hash Declared !!!");
    save_to_json("legacy_proxy_class_hash", &JsonValueType::StringType(legacy_proxy_class_hash.to_string())).unwrap();
    sleep(Duration::from_secs(10)).await;

    let starkgate_proxy_class_hash = declare_contract_middleware(DeclarationInput::LegacyDeclarationInputs(
        String::from(STARKGATE_PROXY_PATH),
        arg_config.rollup_seq_url.clone(),
    ))
    .await;
    log::debug!("Starkgate Proxy Class Hash Declared !!!");
    save_to_json("starkgate_proxy_class_hash", &JsonValueType::StringType(starkgate_proxy_class_hash.to_string()))
        .unwrap();
    sleep(Duration::from_secs(10)).await;

    let erc20_legacy_class_hash = declare_contract_middleware(DeclarationInput::LegacyDeclarationInputs(
        String::from(ERC20_LEGACY_PATH),
        arg_config.rollup_seq_url.clone(),
    ))
    .await;
    log::debug!("ERC20 Legacy Class Hash Declared !!!");
    save_to_json("erc20_legacy_class_hash", &JsonValueType::StringType(erc20_legacy_class_hash.to_string())).unwrap();
    sleep(Duration::from_secs(10)).await;

    let legacy_eth_bridge_class_hash = declare_contract_middleware(DeclarationInput::LegacyDeclarationInputs(
        String::from(LEGACY_BRIDGE_PATH),
        arg_config.rollup_seq_url.clone(),
    ))
    .await;
    log::debug!("Legacy ETH Bridge Class Hash Declared !!!");
    save_to_json("legacy_eth_bridge_class_hash", &JsonValueType::StringType(legacy_eth_bridge_class_hash.to_string()))
        .unwrap();
    sleep(Duration::from_secs(10)).await;

    let oz_account_class_hash = declare_contract_middleware(DeclarationInput::LegacyDeclarationInputs(
        String::from(OZ_ACCOUNT_PATH),
        arg_config.rollup_seq_url.clone(),
    ))
    .await;
    log::debug!("OZ Account Class Hash Declared !!!");
    save_to_json("oz_account_class_hash", &JsonValueType::StringType(oz_account_class_hash.to_string())).unwrap();
    sleep(Duration::from_secs(10)).await;

    log::debug!(">>>> Waiting for block to be mined [/]");
    sleep(Duration::from_secs(10)).await;

    // >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
    // Making temp account for declaration of OZ account Cairo 1 contract
    let account_address_temp =
        deploy_account_using_priv_key(TEMP_ACCOUNT_PRIV_KEY.to_string(), clients.provider_l2(), oz_account_class_hash)
            .await;
    sleep(Duration::from_secs(10)).await;

    let user_account_temp = build_single_owner_account(
        clients.provider_l2(),
        TEMP_ACCOUNT_PRIV_KEY,
        &convert_to_hex(&account_address_temp.to_string()),
        false,
    );
    let oz_account_caio_1_class_hash = declare_contract_middleware(DeclarationInput::DeclarationInputs(
        String::from(OZ_ACCOUNT_SIERRA_PATH),
        String::from(OZ_ACCOUNT_CASM_PATH),
        user_account_temp.clone(),
    ))
    .await;
    save_to_json("oz_account_caio_1_class_hash", &JsonValueType::StringType(oz_account_caio_1_class_hash.to_string()))
        .unwrap();
    sleep(Duration::from_secs(10)).await;
    // >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

    // Using Account Cairo 1 contract
    // >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
    let account_address = deploy_account_using_priv_key(
        arg_config.rollup_priv_key.clone(),
        clients.provider_l2(),
        oz_account_caio_1_class_hash,
    )
    .await;
    sleep(Duration::from_secs(10)).await;
    save_to_json("account_address", &JsonValueType::StringType(account_address.to_string())).unwrap();
    let user_account = build_single_owner_account(
        clients.provider_l2(),
        &arg_config.rollup_priv_key,
        &convert_to_hex(&account_address.to_string()),
        false,
    );
    // >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

    // cairo 1 declaration through account of the user
    let erc_20_class_hash = declare_contract_middleware(DeclarationInput::DeclarationInputs(
        String::from(ERC20_SIERRA_PATH),
        String::from(ERC20_CASM_PATH),
        user_account.clone(),
    ))
    .await;
    log::debug!("ERC20 Class Hash declared !!! : {:?}", erc_20_class_hash);
    save_to_json("erc_20_class_hash", &JsonValueType::StringType(erc_20_class_hash.to_string())).unwrap();
    sleep(Duration::from_secs(10)).await;

    let eth_proxy_address = deploy_proxy_contract(
        &user_account,
        account_address,
        legacy_proxy_class_hash,
        // salt taken from : https://sepolia.starkscan.co/tx/0x06a5a493cf33919e58aa4c75777bffdef97c0e39cac968896d7bee8cc67905a1
        FieldElement::from_str("0x322c2610264639f6b2cee681ac53fa65c37e187ea24292d1b21d859c55e1a78").unwrap(),
        FieldElement::ONE,
    )
    .await;
    log::debug!("ETH Proxy Address : {:?}", eth_proxy_address);
    save_to_json("l2_eth_address_proxy", &JsonValueType::StringType(eth_proxy_address.to_string())).unwrap();
    sleep(Duration::from_secs(10)).await;

    let eth_bridge_proxy_address = deploy_proxy_contract(
        &user_account,
        account_address,
        legacy_proxy_class_hash,
        FieldElement::from_str("0xabcdabcdabcd").unwrap(),
        FieldElement::ZERO,
    )
    .await;
    log::debug!("ETH Bridge Proxy Address : {:?}", eth_bridge_proxy_address);
    save_to_json("ETH_l2_bridge_address_proxy", &JsonValueType::StringType(eth_bridge_proxy_address.to_string()))
        .unwrap();
    sleep(Duration::from_secs(10)).await;

    init_governance_proxy(
        &user_account,
        eth_proxy_address,
        &arg_config.rollup_priv_key,
        "eth_proxy_address : init_governance_proxy",
    )
    .await;
    sleep(Duration::from_secs(10)).await;
    init_governance_proxy(
        &user_account,
        eth_bridge_proxy_address,
        &arg_config.rollup_priv_key,
        "eth_bridge_proxy_address : init_governance_proxy",
    )
    .await;
    sleep(Duration::from_secs(10)).await;

    let token_bridge_proxy_address = deploy_proxy_contract(
        &user_account,
        account_address,
        legacy_proxy_class_hash,
        FieldElement::from_str("0xabcdabcdabcdabcdabcd").unwrap(),
        FieldElement::ZERO,
    )
    .await;
    log::debug!("Token Bridge Proxy Address : {:?}", token_bridge_proxy_address);
    save_to_json("ERC20_l2_bridge_address_proxy", &JsonValueType::StringType(token_bridge_proxy_address.to_string()))
        .unwrap();
    sleep(Duration::from_secs(10)).await;
    init_governance_proxy(
        &user_account,
        token_bridge_proxy_address,
        &arg_config.rollup_priv_key,
        "token_bridge_proxy_address : init_governance_proxy",
    )
    .await;
    sleep(Duration::from_secs(10)).await;

    (
        erc_20_class_hash,
        legacy_eth_bridge_class_hash,
        account_address,
        eth_proxy_address,
        eth_bridge_proxy_address,
        token_bridge_proxy_address,
        FieldElement::ZERO,
        legacy_proxy_class_hash,
        starkgate_proxy_class_hash,
        erc20_legacy_class_hash,
    )
}

pub(crate) enum DeclarationInput<'a> {
    // inputs : sierra_path, casm_path
    DeclarationInputs(String, String, RpcAccount<'a>),
    // input : artifact_path
    LegacyDeclarationInputs(String, String),
}

#[allow(private_interfaces)]
pub async fn declare_contract_middleware(input: DeclarationInput<'_>) -> FieldElement {
    match input {
        DeclarationInput::DeclarationInputs(sierra_path, casm_path, account) => {
            let (class_hash, sierra) = account.declare_contract_params_sierra(&sierra_path, &casm_path);

            account
                .declare(Arc::new(sierra.clone()), class_hash)
                .send()
                .await
                .expect("Error in declaring the contract using Cairo 1 declaration using the provided account !!!");
            sierra.class_hash()
        }
        DeclarationInput::LegacyDeclarationInputs(artifact_path, url) => {
            let contract_artifact: ContractClassV0Inner = serde_json::from_reader(
                std::fs::File::open(env!("CARGO_MANIFEST_DIR").to_owned() + "/" + &artifact_path).unwrap(),
            )
            .unwrap();

            let contract_abi_artifact: LegacyContractClass = serde_json::from_reader(
                std::fs::File::open(env!("CARGO_MANIFEST_DIR").to_owned() + "/" + &artifact_path).unwrap(),
            )
            .unwrap();

            let empty_vector_stark_hash: Vec<StarkHash> = Vec::new();
            let empty_array: [u8; 32] = [0; 32];

            let class_info = contract_artifact.clone();

            let program = class_info.program;
            let encoded_p = program.encode();
            let entry_points_by_type = class_info.entry_points_by_type;

            let declare_txn: DeclareTransactionV0V1 = DeclareTransactionV0V1 {
                max_fee: Fee(0),
                signature: TransactionSignature(empty_vector_stark_hash),
                nonce: Nonce(StarkFelt(empty_array)),
                class_hash: ClassHash(StarkHash { 0: contract_abi_artifact.class_hash().unwrap().to_bytes_be() }),
                sender_address: ContractAddress(PatriciaKey(StarkHash { 0: FieldElement::ONE.to_bytes_be() })),
            };
            let abi_length = contract_abi_artifact.abi.len();

            let params: CustomDeclareV0Transaction = CustomDeclareV0Transaction {
                declare_transaction: declare_txn,
                program_vec: encoded_p,
                entrypoints: entry_points_by_type,
                abi_length,
            };

            let json_body = &json!({
                "jsonrpc": "2.0",
                "method": "madara_declareV0",
                "params": [params],
                "id": 4
            });

            let req_client = reqwest::Client::new();
            let raw_txn_rpc = req_client.post(url).json(json_body).send().await;

            match raw_txn_rpc {
                Ok(val) => {
                    log::debug!(
                        "Txn Sent Successfully : {:?}",
                        val.json::<RpcResult<DeclareV0Result>>().await.unwrap()
                    );
                    log::debug!("Declare Success : {:?}", contract_abi_artifact.class_hash().unwrap());
                }
                Err(err) => {
                    log::debug!("Error : Error sending the transaction using RPC");
                    log::debug!("{:?}", err);
                }
            }

            contract_abi_artifact.class_hash().unwrap()
        }
    }
}

async fn deploy_account_using_priv_key(
    priv_key: String,
    provider: &JsonRpcClient<HttpTransport>,
    oz_account_class_hash: FieldElement,
) -> FieldElement {
    let chain_id = provider.chain_id().await.unwrap();
    let signer = Arc::new(LocalWallet::from_signing_key(SigningKey::from_secret_scalar(
        FieldElement::from_hex_be(&priv_key).unwrap(),
    )));
    log::debug!(">>>>> signer : {:?}", signer);
    let mut oz_account_factory =
        OpenZeppelinAccountFactory::new(oz_account_class_hash, chain_id, signer, provider).await.unwrap();
    oz_account_factory.set_block_id(BlockId::Tag(BlockTag::Pending));

    let deploy_txn = oz_account_factory.deploy(FieldElement::ZERO);
    let account_address = deploy_txn.address();
    log::debug!("OZ Account Deployed : {:?}", account_address);
    save_to_json("account_address", &JsonValueType::StringType(account_address.to_string())).unwrap();

    let sent_txn = deploy_txn.send().await.expect("Error in deploying the OZ account");

    log::debug!("deploy account txn_hash : {:?}", sent_txn.transaction_hash);
    wait_for_transaction(provider, sent_txn.transaction_hash, "deploy_account_using_priv_key").await.unwrap();

    account_address
}

async fn deploy_proxy_contract(
    account: &RpcAccount<'_>,
    account_address: FieldElement,
    class_hash: FieldElement,
    salt: FieldElement,
    deploy_from_zero: FieldElement,
) -> FieldElement {
    let txn = account
        .invoke_contract(
            account_address,
            "deploy_contract",
            vec![class_hash, salt, deploy_from_zero, FieldElement::ONE, FieldElement::ZERO],
            None,
        )
        .send()
        .await
        .expect("Error deploying the contract proxy.");

    wait_for_transaction(account.provider(), txn.transaction_hash, "deploy_proxy_contract : deploy_contract")
        .await
        .unwrap();

    log::debug!(">>>>> txn hash (proxy deployment) : {:?}", txn.transaction_hash);

    let deployed_address_event = get_contract_address_from_deploy_tx(account.provider(), &txn).await.unwrap();
    log::debug!(">>>>> [IMP] >>>>> : {:?}", deployed_address_event);

    deployed_address_event
}

async fn init_governance_proxy(account: &RpcAccount<'_>, contract_address: FieldElement, p_key: &str, tag: &str) {
    let txn = invoke_contract(
        account.provider(),
        contract_address,
        "init_governance",
        vec![],
        p_key,
        &convert_to_hex(&account.address().to_string()),
    )
    .await;
    wait_for_transaction(account.provider(), txn.transaction_hash, tag).await.unwrap();
}
