use std::sync::Arc;

use anyhow::Context;
use ethers::types::U256;
use hex::encode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use starknet_accounts::{Account, AccountFactory, ConnectedAccount, OpenZeppelinAccountFactory, SingleOwnerAccount};
use starknet_api::hash::{pedersen_hash_array, StarkFelt, StarkHash};
use starknet_core::types::contract::legacy::LegacyContractClass;
use starknet_core::types::{BlockId, BlockTag, CompressedLegacyContractClass, DeclareTransactionResult, FunctionCall};
use starknet_core::utils::get_selector_from_name;
use starknet_ff::FieldElement;
use starknet_providers::jsonrpc::HttpTransport;
use starknet_providers::{JsonRpcClient, Provider};
use starknet_signers::{LocalWallet, SigningKey};

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
) -> anyhow::Result<RpcAccount<'a>> {
    let signer = LocalWallet::from(SigningKey::from_secret_scalar(
        FieldElement::from_hex_be(private_key).context("Parsing private key")?,
    ));
    let account_address = FieldElement::from_hex_be(account_address)
        .with_context(|| format!("Invalid contract address: {account_address}"))?;
    let execution_encoding = if is_legacy {
        starknet_accounts::ExecutionEncoding::Legacy
    } else {
        starknet_accounts::ExecutionEncoding::New
    };

    let chain_id = rpc.chain_id().await.context("Getting the chain id from RPC")?;
    Ok(SingleOwnerAccount::new(rpc, signer, account_address, chain_id, execution_encoding))
}

pub async fn read_erc20_balance(
    rpc: &JsonRpcClient<HttpTransport>,
    contract_address: FieldElement,
    account_address: FieldElement,
) -> anyhow::Result<Vec<FieldElement>> {
    rpc.call(
        FunctionCall {
            contract_address,
            entry_point_selector: get_selector_from_name("balanceOf")?,
            calldata: vec![account_address],
        },
        BlockId::Tag(BlockTag::Latest),
    )
    .await
    .with_context(|| {
        format!("Reading the erc20 balance for {account_address:#x} on ERC20 contract {contract_address:#x}")
    })
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
        StarkFelt(config_hash_version.to_bytes_be()),
        StarkFelt(chain_id.to_bytes_be()),
        StarkFelt(fee_token_address.to_bytes_be()),
    ])
}

pub fn get_bridge_init_configs(config: &CliArgs) -> anyhow::Result<(FieldElement, StarkHash)> {
    let program_hash =
        FieldElement::from_hex_be(config.sn_os_program_hash.as_str()).context("Parsing sn_os_program_hash")?;
    let config_hash = generate_config_hash(
        FieldElement::from_hex_be(&encode(config.config_hash_version.as_str()))
            .context("Parsing config_hash_version")?,
        FieldElement::from_hex_be(&encode(config.app_chain_id.as_str())).context("Parsing app_chain_id")?,
        FieldElement::from_hex_be(config.fee_token_address.as_str()).context("Parsing fee_token_address")?,
    );
    Ok((program_hash, config_hash))
}

/// Broadcasted declare contract transaction v0.
#[derive(Debug, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BroadcastedDeclareTransactionV0 {
    /// The address of the account contract sending the declaration transaction
    pub sender_address: FieldElement,
    /// The maximal fee that can be charged for including the transaction
    pub max_fee: FieldElement,
    /// Signature
    pub signature: Vec<FieldElement>,
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
pub async fn declare_contract(input: DeclarationInput<'_>) -> anyhow::Result<FieldElement> {
    match input {
        DeclarationInputs(sierra_path, casm_path, account) => {
            let (class_hash, sierra) = account
                .declare_contract_params_sierra(&sierra_path, &casm_path)
                .context("Creating declare sierra transaction")?;

            account
                .declare(Arc::new(sierra.clone()), class_hash)
                .send()
                .await
                .context("Error while declaring the contract using Cairo 1 declaration using the provided account")?;
            Ok(sierra.class_hash())
        }
        LegacyDeclarationInputs(artifact_path, url) => {
            let path = format!("{}/{}", env!("CARGO_MANIFEST_DIR").to_owned(), &artifact_path);
            let contract_abi_artifact_temp: LegacyContractClass =
                serde_json::from_reader(std::fs::File::open(&path).with_context(|| format!("Opening file {}", path))?)
                    .with_context(|| format!("Reading file {}", path))?;

            let contract_abi_artifact =
                contract_abi_artifact_temp.clone().compress().context("Failed to compress the contract class")?;

            let params: BroadcastedDeclareTransactionV0 = BroadcastedDeclareTransactionV0 {
                sender_address: FieldElement::from_hex_be("0x1").expect("Parsing constant"),
                max_fee: FieldElement::from(482250u128),
                signature: Vec::new(),
                contract_class: Arc::new(contract_abi_artifact),
                is_query: false,
            };

            let json_body = &json!({
                "jsonrpc": "2.0",
                "method": "madara_addDeclareTransactionV0",
                "params": [params],
                "id": 4
            });

            let req_client = reqwest::Client::new();
            let raw_res: DeclareTransactionResult = req_client
                .post(url)
                .json(json_body)
                .send()
                .await
                .context("Declaring contract via RPC")?
                .json()
                .await
                .context("Parsing json response from RPC")?;
            log::debug!("🚧 Txn Sent Successfully : {:?}", raw_res);

            contract_abi_artifact_temp.class_hash().context("Getting class hash from contract ABI")
        }
    }
}

pub(crate) async fn deploy_account_using_priv_key(
    priv_key: String,
    provider: &JsonRpcClient<HttpTransport>,
    oz_account_class_hash: FieldElement,
) -> anyhow::Result<FieldElement> {
    let chain_id = provider.chain_id().await.context("Parsing chain id")?;
    let signer = Arc::new(LocalWallet::from_signing_key(SigningKey::from_secret_scalar(
        FieldElement::from_hex_be(&priv_key).context("Parsing private key")?,
    )));
    log::trace!("signer : {:?}", signer);
    let mut oz_account_factory = OpenZeppelinAccountFactory::new(oz_account_class_hash, chain_id, signer, provider)
        .await
        .context("Making OZ factory")?;
    oz_account_factory.set_block_id(BlockId::Tag(BlockTag::Pending));

    let deploy_txn = oz_account_factory.deploy(FieldElement::ZERO);
    let account_address = deploy_txn.address();
    log::trace!("OZ Account Deployed : {:?}", account_address);
    save_to_json("account_address", &JsonValueType::StringType(account_address.to_string()))
        .context("Saving deployed OZ account address to json")?;

    let sent_txn = deploy_txn.send().await.context("Deploying the OZ account")?;

    log::trace!("deploy account txn_hash : {:?}", sent_txn.transaction_hash);
    wait_for_transaction(provider, sent_txn.transaction_hash, "deploy_account_using_priv_key")
        .await
        .context("Waiting for the OZ account to be deployed")?;

    Ok(account_address)
}

pub(crate) async fn deploy_proxy_contract(
    account: &RpcAccount<'_>,
    account_address: FieldElement,
    class_hash: FieldElement,
    salt: FieldElement,
    deploy_from_zero: FieldElement,
) -> anyhow::Result<FieldElement> {
    let txn = account
        .invoke_contract(
            account_address,
            "deploy_contract",
            vec![class_hash, salt, deploy_from_zero, FieldElement::ONE, FieldElement::ZERO],
            None,
        )
        .context("Making deploy_contract transaction")?
        .send()
        .await
        .context("Error deploying the contract proxy")?;

    wait_for_transaction(account.provider(), txn.transaction_hash, "deploy_proxy_contract : deploy_contract")
        .await
        .context("Waiting for the proxy contract deployment")?;

    log::trace!("txn hash (proxy deployment) : {:?}", txn.transaction_hash);

    let deployed_address = get_contract_address_from_deploy_tx(account.provider(), &txn)
        .await
        .context("Getting contract address from deploy transaction")?;
    log::trace!("[IMP] Event : {:?}", deployed_address);

    Ok(deployed_address)
}

pub(crate) async fn init_governance_proxy(
    account: &'_ RpcAccount<'_>,
    contract_address: FieldElement,
    tag: &str,
) -> anyhow::Result<()> {
    let txn = invoke_contract(contract_address, "init_governance", vec![], account)
        .await
        .context("Making init_governance transaction")?;
    wait_for_transaction(account.provider(), txn.transaction_hash, tag)
        .await
        .context("Waiting for init_governance transaction to settle")?;
    Ok(())
}
