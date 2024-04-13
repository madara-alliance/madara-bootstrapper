use ethers::addressbook::Address;
use starknet_ff::FieldElement;
use starknet_providers::jsonrpc::HttpTransport;
use starknet_providers::JsonRpcClient;
use crate::bridge::helpers::account_actions::{AccountActions, get_transaction_receipt};
use crate::bridge::helpers::deploy_utils::build_single_owner_account;
use std::time::Duration;
use starknet_core::types::{InvokeTransactionResult};
use starknet_core::types::MaybePendingTransactionReceipt::{PendingReceipt, Receipt};
use tokio::time::sleep;

pub async fn invoke_contract(
    rpc_provider: &JsonRpcClient<HttpTransport>,
    contract: FieldElement,
    method: &str,
    calldata: Vec<FieldElement>,
    priv_key: &str,
    address: &str
) -> InvokeTransactionResult {
    let account = build_single_owner_account(&rpc_provider, priv_key, address, false);

    let txn_res = account.invoke_contract(contract, method, calldata, None).send().await.expect("Error in invoking the contract !!");

    wait_for_transaction(rpc_provider,txn_res.transaction_hash).await.unwrap();

    txn_res
}

pub fn pad_bytes(address: Address) -> Vec<u8> {
    let address_bytes = address.as_bytes();
    let mut padded_address_bytes = Vec::with_capacity(32);
    padded_address_bytes.extend(vec![0u8; 32 - address_bytes.len()]);
    padded_address_bytes.extend_from_slice(address_bytes);
    padded_address_bytes
}

pub async fn wait_for_transaction(provider_l2: &JsonRpcClient<HttpTransport>,transaction_hash: FieldElement) -> Result<(), anyhow::Error> {

    let transaction_receipt = get_transaction_receipt(provider_l2, transaction_hash).await;

    let transaction_status = transaction_receipt.ok().unwrap();

    match transaction_status {
        Receipt(..) => {
            Ok(())
        },
        PendingReceipt(..) => {
            log::debug!(">>>> waiting for transaction : {:?}", transaction_hash);
            sleep(Duration::from_secs(2)).await;
            Box::pin(wait_for_transaction(provider_l2, transaction_hash)).await
        }
    }
}