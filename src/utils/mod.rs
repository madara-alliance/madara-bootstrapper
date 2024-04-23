use std::path::Path;
use std::time::Duration;
use std::{fs, io};

use ethers::addressbook::Address;
use ethers::types::U256;
use num_bigint::BigUint;
use serde_json::{Map, Value};
use starknet_api::hash::StarkFelt;
use starknet_core::types::InvokeTransactionResult;
use starknet_core::types::MaybePendingTransactionReceipt::{PendingReceipt, Receipt};
use starknet_ff::FieldElement;
use starknet_providers::jsonrpc::HttpTransport;
use starknet_providers::JsonRpcClient;
use tokio::time::sleep;

use crate::bridge::helpers::account_actions::{get_transaction_receipt, AccountActions};
use crate::contract_clients::utils::build_single_owner_account;

pub mod constants;

pub async fn invoke_contract(
    rpc_provider: &JsonRpcClient<HttpTransport>,
    contract: FieldElement,
    method: &str,
    calldata: Vec<FieldElement>,
    priv_key: &str,
    address: &str,
) -> InvokeTransactionResult {
    let account = build_single_owner_account(rpc_provider, priv_key, address, false);

    let txn_res = account
        .invoke_contract(contract, method, calldata, None)
        .send()
        .await
        .expect("Error in invoking the contract !!");

    wait_for_transaction(rpc_provider, txn_res.transaction_hash).await.unwrap();

    txn_res
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
) -> Result<(), anyhow::Error> {
    let transaction_receipt = get_transaction_receipt(provider_l2, transaction_hash).await;

    let transaction_status = transaction_receipt.ok().unwrap();

    match transaction_status {
        Receipt(..) => Ok(()),
        PendingReceipt(..) => {
            log::trace!("⏳ waiting for transaction : {:?}", transaction_hash);
            sleep(Duration::from_secs(2)).await;
            Box::pin(wait_for_transaction(provider_l2, transaction_hash)).await
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

pub fn save_to_json(key: &str, value: &JsonValueType) -> Result<(), io::Error> {
    let file_path: &str = "./data/addresses.json";
    let data = fs::read_to_string(file_path);
    let mut json: Map<String, Value> = match data {
        Ok(content) => serde_json::from_str(&content).unwrap_or_else(|_| Map::new()),
        Err(_) => Map::new(),
    };

    match value {
        JsonValueType::EthAddress(x) => {
            json.insert(key.to_string(), serde_json::json!(x));
        }
        JsonValueType::StringType(x) => {
            json.insert(key.to_string(), serde_json::json!(convert_to_hex(x)));
        }
    }

    let updated_json = serde_json::to_string_pretty(&json)?;

    // Ensure the directory exists before writing the file
    if let Some(dir_path) = Path::new(file_path).parent() {
        fs::create_dir_all(dir_path)?;
    }

    fs::write(file_path, updated_json)?;

    Ok(())
}

fn convert_to_hex(address: &str) -> String {
    let big_uint = address.parse::<BigUint>().map_err(|_| "Invalid number");
    big_uint.expect("error converting decimal string ---> hex string").to_str_radix(16)
}
