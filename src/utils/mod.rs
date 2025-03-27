use std::path::Path;
use std::{fs, io};

use ethers::abi::Address;
use ethers::types::U256;
use num_bigint::BigUint;
use serde_json::{Map, Value};
use starknet::accounts::ConnectedAccount;
use starknet::core::types::{Felt, InvokeTransactionResult, TransactionReceipt};
use starknet_core::types::TransactionReceiptWithBlockInfo;
use starknet_providers::jsonrpc::HttpTransport;
use starknet_providers::JsonRpcClient;

use crate::contract_clients::utils::RpcAccount;
use crate::helpers::account_actions::{get_transaction_receipt, AccountActions};

pub mod banner;
pub mod constants;

pub async fn invoke_contract<'a>(
    contract: Felt,
    method: &str,
    calldata: Vec<Felt>,
    account: &RpcAccount<'a>,
) -> InvokeTransactionResult {
    let txn_res =
        account.invoke_contract(contract, method, calldata, None).send().await.expect("Error in invoking the contract");

    wait_for_transaction(account.provider(), txn_res.transaction_hash, "invoking_contract").await.unwrap();

    txn_res
}

pub fn pad_bytes(address: Address) -> Vec<u8> {
    let address_bytes = address.as_bytes();
    let mut padded_address_bytes = Vec::with_capacity(32);
    padded_address_bytes.extend(vec![0u8; 32 - address_bytes.len()]);
    padded_address_bytes.extend_from_slice(address_bytes);
    padded_address_bytes
}

pub fn hexstring_to_address(hex: &str) -> ethers::abi::Address {
    let hexstring = format!("0x{:0>40}", hex.strip_prefix("0x").unwrap_or(hex));
    Address::from_slice(&hexstring.as_bytes()[2..])
}

pub async fn wait_for_transaction(
    provider_l2: &JsonRpcClient<HttpTransport>,
    transaction_hash: Felt,
    tag: &str,
) -> Result<(), anyhow::Error> {
    let transaction_receipt = get_transaction_receipt(provider_l2, transaction_hash).await;

    let transaction_status = transaction_receipt.ok().unwrap();

    match transaction_status {
        TransactionReceiptWithBlockInfo { receipt: TransactionReceipt::Invoke(receipt), .. } => {
            log::trace!("txn : {:?} : {:?}", tag, receipt);
        }
        TransactionReceiptWithBlockInfo { receipt: TransactionReceipt::DeployAccount(receipt), .. } => {
            let contract_address = receipt.contract_address;
            log::trace!("txn : {:?} : {:?}", tag, contract_address);
        }
        _ => {
            log::error!("Transaction status: {:?}", transaction_status);
            panic!("Transaction failed");
        }
    };

    Ok(())
}

pub fn convert_felt_to_u256(felt: Felt) -> U256 {
    U256::from_big_endian(&felt.to_bytes_be())
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

pub fn convert_to_hex(address: &str) -> String {
    let big_uint = address.parse::<BigUint>().map_err(|_| "Invalid number");
    let hex = big_uint.expect("error converting decimal string ---> hex string").to_str_radix(16);
    "0x".to_string() + &hex
}
