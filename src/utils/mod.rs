use std::fs;
use std::fs::File;
use std::path::Path;
use std::time::Duration;

use anyhow::Context;
use ethers::addressbook::Address;
use ethers::types::U256;
use num_bigint::BigUint;
use serde_json::{Map, Value};
use starknet_accounts::ConnectedAccount;
use starknet_api::hash::StarkFelt;
use starknet_core::types::InvokeTransactionResult;
use starknet_core::types::MaybePendingTransactionReceipt::{PendingReceipt, Receipt};
use starknet_ff::FieldElement;
use starknet_providers::jsonrpc::HttpTransport;
use starknet_providers::JsonRpcClient;
use tokio::time::sleep;

use crate::contract_clients::utils::RpcAccount;
use crate::helpers::account_actions::{get_transaction_receipt, AccountActions};

pub mod banner;
pub mod constants;

pub async fn invoke_contract<'a>(
    contract: FieldElement,
    method: &str,
    calldata: Vec<FieldElement>,
    account: &RpcAccount<'a>,
) -> anyhow::Result<InvokeTransactionResult> {
    let txn_res = account
        .invoke_contract(contract, method, calldata, None)
        .with_context(|| format!("Making invoke transaction for contract {contract:#x}, method {method}"))?
        .send()
        .await
        .with_context(|| format!("Sending invoke transaction for contract {contract:#x}, method {method}"))?;

    wait_for_transaction(account.provider(), txn_res.transaction_hash, "invoking_contract").await.with_context(
        || format!("Waiting for invoke transaction for contract {contract:#x}, method {method} to settle"),
    )?;

    Ok(txn_res)
}

pub fn pad_bytes(address: Address) -> Vec<u8> {
    let address_bytes = address.as_bytes();
    let mut padded_address_bytes = Vec::with_capacity(32);
    padded_address_bytes.extend(vec![0u8; 32 - address_bytes.len()]);
    padded_address_bytes.extend_from_slice(address_bytes);
    padded_address_bytes
}

pub async fn wait_for_transaction(
    provider_l2: &JsonRpcClient<HttpTransport>,
    transaction_hash: FieldElement,
    tag: &str,
) -> anyhow::Result<()> {
    let transaction_receipt = get_transaction_receipt(provider_l2, transaction_hash)
        .await
        .with_context(|| format!("Getting transaction receipt for transaction hash {transaction_hash:#x}"))?;

    match transaction_receipt {
        Receipt(transaction_receipt) => {
            log::trace!("txn : {:?} : {:?}", tag, transaction_receipt);
            Ok(())
        }
        PendingReceipt(..) => {
            log::trace!("⏳ waiting for transaction : {:?}", transaction_hash);
            sleep(Duration::from_secs(2)).await;
            Box::pin(wait_for_transaction(provider_l2, transaction_hash, "")).await
        }
    }
}

pub fn convert_felt_to_u256(felt: StarkFelt) -> U256 {
    U256::from_big_endian(felt.bytes())
}

pub enum JsonValueType {
    EthAddress(Address),
    StringType(String),
}

pub fn save_to_json(key: &str, value: &JsonValueType) -> anyhow::Result<()> {
    let file_path = Path::new("./data/addresses.json");

    let mut json: Map<String, Value> = if file_path.exists() {
        let file = File::open(file_path).with_context(|| format!("Opening file {}", file_path.display()))?;
        serde_json::from_reader(file)
            .with_context(|| format!("Reading and deserializing file {}", file_path.display()))?
    } else {
        Default::default()
    };

    match value {
        JsonValueType::EthAddress(x) => {
            json.insert(key.to_string(), serde_json::json!(x));
        }
        JsonValueType::StringType(x) => {
            json.insert(key.to_string(), serde_json::json!(convert_to_hex(x)?));
        }
    }

    let updated_json = serde_json::to_string_pretty(&json).context("Serializing to json")?;

    // Ensure the directory exists before writing the file
    if let Some(dir_path) = Path::new(file_path).parent() {
        fs::create_dir_all(dir_path)
            .with_context(|| format!("Creating the parent directories for file {}", file_path.display()))?;
    }

    fs::write(file_path, updated_json).with_context(|| format!("Writing the json file to {}", file_path.display()))?;

    Ok(())
}

pub fn convert_to_hex(address: &str) -> anyhow::Result<String> {
    let big_uint = address.parse::<BigUint>().with_context(|| format!("Parsing address {address}"))?;
    let hex = big_uint.to_str_radix(16);
    Ok(format!("0x{hex}"))
}
