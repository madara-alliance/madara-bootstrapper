use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use blockifier::execution::contract_class::{ClassInfo, ContractClass, ContractClassV0, ContractClassV0Inner};
use blockifier::transaction::transactions::DeclareTransaction;
use cairo_vm::types::program::Program;
use clap::Parser;
use indexmap::IndexMap;
use log::log;
use parity_scale_codec::Encode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use starknet_accounts::{Account, AccountFactory, ConnectedAccount, OpenZeppelinAccountFactory};
use starknet_api::core::{ClassHash, ContractAddress, Nonce, PatriciaKey};
use starknet_api::deprecated_contract_class::{EntryPoint, EntryPointType};
use starknet_api::hash::{StarkFelt, StarkHash};
use starknet_api::transaction::{
    DeclareTransaction as DeclareTransactionEnum, DeclareTransactionV0V1, Fee, TransactionHash, TransactionSignature,
};
use starknet_core::types::contract::legacy::LegacyContractClass;
use starknet_core::types::{BlockId, BlockTag};
use starknet_ff::FieldElement;
use starknet_providers::jsonrpc::HttpTransport;
use starknet_providers::{JsonRpcClient, Provider};
use starknet_signers::{LocalWallet, SigningKey};
use tokio::time::sleep;

use crate::bridge::helpers::account_actions::{get_contract_address_from_deploy_tx, AccountActions};
use crate::contract_clients::config::Config;
// use crate::contract_clients::subxt_funcs::toggle_fee;
use crate::contract_clients::utils::{build_single_owner_account, RpcAccount};
use crate::utils::constants::{
    ERC20_CASM_PATH, ERC20_SIERRA_PATH, LEGACY_BRIDGE_PATH, LEGACY_BRIDGE_PROGRAM_PATH, OZ_ACCOUNT_PATH,
    OZ_ACCOUNT_PROGRAM_PATH, PROXY_PATH, PROXY_PROGRAM_PATH,
};
use crate::utils::mapper::map_entrypoint_selector;
use crate::utils::{invoke_contract, save_to_json, wait_for_transaction, JsonValueType, convert_to_hex};
use crate::CliArgs;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct CustomDeclareV0Transaction {
    pub declare_transaction: DeclareTransactionV0V1,
    pub program_vec: Vec<u8>,
    pub entrypoints: IndexMap<EntryPointType, Vec<EntryPoint>>,
    pub abi_length: usize
}

pub async fn init_and_deploy_eth_and_account(
    clients: &Config,
    arg_config: &CliArgs,
) -> (FieldElement, FieldElement, FieldElement, FieldElement, FieldElement, FieldElement, FieldElement) {
    // toggle_fee(true).await.expect("Error in disabling the fee on configured app-chain");

    let legacy_eth_bridge_class_hash = declare_contract_middleware(DeclarationInput::LegacyDeclarationInputs(
        String::from(LEGACY_BRIDGE_PATH),
        String::from(LEGACY_BRIDGE_PROGRAM_PATH),
        arg_config.rollup_seq_url.clone(),
    ))
    .await;
    log::debug!("Legacy ETH Bridge Class Hash Declared !!!");
    save_to_json("legacy_eth_bridge_class_hash", &JsonValueType::StringType(legacy_eth_bridge_class_hash.to_string()))
        .unwrap();
    sleep(Duration::from_secs(10)).await;
    
    let oz_account_class_hash = declare_contract_middleware(DeclarationInput::LegacyDeclarationInputs(
        String::from(OZ_ACCOUNT_PATH),
        String::from(OZ_ACCOUNT_PROGRAM_PATH),
        arg_config.rollup_seq_url.clone(),
    ))
    .await;
    log::debug!("OZ Account Class Hash Declared !!!");
    save_to_json("oz_account_class_hash", &JsonValueType::StringType(oz_account_class_hash.to_string())).unwrap();
    sleep(Duration::from_secs(10)).await;
    
    let proxy_class_hash = declare_contract_middleware(DeclarationInput::LegacyDeclarationInputs(
        String::from(PROXY_PATH),
        String::from(PROXY_PROGRAM_PATH),
        arg_config.rollup_seq_url.clone(),
    ))
    .await;
    log::debug!("Proxy Class Hash Declared !!!");
    save_to_json("proxy_class_hash", &JsonValueType::StringType(proxy_class_hash.to_string())).unwrap();

    log::debug!(">>>> Waiting for block to be mined [/]");
    sleep(Duration::from_secs(10)).await;
    
    let account_address =
        deploy_account_using_priv_key(arg_config.rollup_priv_key.clone(), clients.provider_l2(), oz_account_class_hash)
            .await;
    save_to_json("account_address", &JsonValueType::StringType(account_address.to_string())).unwrap();
    sleep(Duration::from_secs(10)).await;
    
    let user_account = build_single_owner_account(
        clients.provider_l2(),
        &*arg_config.rollup_priv_key,
        &convert_to_hex(&account_address.to_string()),
        false,
    );
    
    // cairo 1 declarations through account
    let erc_20_class_hash = declare_contract_middleware(DeclarationInput::DeclarationInputs(
        String::from(ERC20_SIERRA_PATH),
        String::from(ERC20_CASM_PATH),
        user_account.clone(),
    ))
    .await;
    log::debug!("ERC20 Class Hash declared !!! : {:?}", erc_20_class_hash);
    save_to_json("erc_20_class_hash", &JsonValueType::StringType(erc_20_class_hash.to_string())).unwrap();

    let eth_proxy_address = deploy_proxy_contract(
        &user_account,
        account_address,
        proxy_class_hash,
        // salt taken from : https://sepolia.starkscan.co/tx/0x06a5a493cf33919e58aa4c75777bffdef97c0e39cac968896d7bee8cc67905a1
        FieldElement::from_str("0x322c2610264639f6b2cee681ac53fa65c37e187ea24292d1b21d859c55e1a78").unwrap(),
        FieldElement::ONE,
    )
    .await;
    log::debug!("ETH Proxy Address : {:?}", eth_proxy_address);
    save_to_json("l2_eth_address_proxy", &JsonValueType::StringType(eth_proxy_address.to_string())).unwrap();

    let eth_bridge_proxy_address = deploy_proxy_contract(
        &user_account,
        account_address,
        proxy_class_hash,
        FieldElement::from_str("0xabcdabcd").unwrap(),
        FieldElement::ZERO,
    )
    .await;
    log::debug!("ETH Bridge Proxy Address : {:?}", eth_proxy_address);
    save_to_json("ETH_l2_bridge_address_proxy", &JsonValueType::StringType(eth_proxy_address.to_string())).unwrap();

    init_governance_proxy(&user_account, eth_proxy_address, &arg_config.rollup_priv_key).await;
    init_governance_proxy(&user_account, eth_bridge_proxy_address, &arg_config.rollup_priv_key).await;

    let token_bridge_proxy_address = deploy_proxy_contract(
        &user_account,
        account_address,
        proxy_class_hash,
        FieldElement::from_str("0xabcdabcd").unwrap(),
        FieldElement::ZERO,
    )
    .await;
    log::debug!("Token Bridge Proxy Address : {:?}", eth_proxy_address);
    save_to_json("ERC20_l2_bridge_address_proxy", &JsonValueType::StringType(token_bridge_proxy_address.to_string()))
        .unwrap();

    (
        erc_20_class_hash,
        legacy_eth_bridge_class_hash,
        account_address,
        eth_proxy_address,
        eth_bridge_proxy_address,
        token_bridge_proxy_address,
        proxy_class_hash,
    )
}

enum DeclarationInput<'a> {
    // inputs : sierra_path, casm_path
    DeclarationInputs(String, String, RpcAccount<'a>),
    // input : artifact_path
    LegacyDeclarationInputs(String, String, String),
}

async fn declare_contract_middleware(input: DeclarationInput<'_>) -> FieldElement {
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
        DeclarationInput::LegacyDeclarationInputs(artifact_path, program_artifact_path, url) => {
            let contract_artifact: ContractClassV0Inner = serde_json::from_reader(
                std::fs::File::open(env!("CARGO_MANIFEST_DIR").to_owned() + "/" + &artifact_path).unwrap(),
            )
            .unwrap();

            let contract_abi_artifact: LegacyContractClass = serde_json::from_reader(
                std::fs::File::open(env!("CARGO_MANIFEST_DIR").to_owned() + "/" + &artifact_path).unwrap(),
            ).unwrap();
            
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
                sender_address: ContractAddress(PatriciaKey { 0: StarkHash { 0: FieldElement::ONE.to_bytes_be() } }),
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
                Result::Ok(val) => {
                    log::debug!("Txn Sent Successfully : {:?}", val);
                    log::debug!("Declare Success : {:?}", contract_abi_artifact.class_hash().unwrap());
                }
                Result::Err(err) => {
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
    wait_for_transaction(provider, sent_txn.transaction_hash).await.unwrap();

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
            vec![class_hash, salt, FieldElement::ONE, FieldElement::ZERO, deploy_from_zero],
            None,
        )
        .send()
        .await
        .expect("Error deploying the contract proxy.");

    wait_for_transaction(account.provider(), txn.transaction_hash).await.unwrap();
    get_contract_address_from_deploy_tx(account.provider(), &txn).await.unwrap()
}

async fn init_governance_proxy(account: &RpcAccount<'_>, contract_address: FieldElement, p_key: &str) {
    let txn = invoke_contract(
        account.provider(),
        contract_address,
        "init_governance",
        vec![],
        p_key,
        &account.address().to_string(),
    )
    .await;
    wait_for_transaction(account.provider(), txn.transaction_hash).await.unwrap();
}
