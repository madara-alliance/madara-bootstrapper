use std::future::Future;

use assert_matches::assert_matches;
use async_trait::async_trait;
use starknet_accounts::{Account, Call, Execution, SingleOwnerAccount};
use starknet_api::core::{calculate_contract_address, ClassHash, ContractAddress};
use starknet_api::transaction::{Calldata, ContractAddressSalt};
use starknet_core::types::contract::legacy::LegacyContractClass;
use starknet_core::types::contract::{CompiledClass, SierraClass};
use starknet_core::types::{
    FlattenedSierraClass, InvokeTransactionResult, MaybePendingTransactionReceipt, TransactionReceipt,
};
use starknet_core::utils::get_selector_from_name;
use starknet_ff::FieldElement;
use starknet_providers::jsonrpc::HttpTransport;
use starknet_providers::{JsonRpcClient, Provider, ProviderError};
use starknet_signers::LocalWallet;

use crate::contract_clients::utils::RpcAccount;
use crate::utils::constants::MAX_FEE_OVERRIDE;
use crate::utils::wait_for_transaction;

pub struct U256 {
    pub high: FieldElement,
    pub low: FieldElement,
}

pub type TransactionExecution<'a> = Execution<'a, RpcAccount<'a>>;

#[async_trait]
pub trait AccountActions {
    fn invoke_contract(
        &self,
        address: FieldElement,
        method: &str,
        calldata: Vec<FieldElement>,
        nonce: Option<u64>,
    ) -> TransactionExecution;

    fn declare_contract_params_sierra(
        &self,
        path_to_sierra: &str,
        path_to_casm: &str,
    ) -> (FieldElement, FlattenedSierraClass);
    fn declare_contract_params_legacy(&self, path_to_compiled_contract: &str) -> LegacyContractClass;
}

impl AccountActions for SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet> {
    fn invoke_contract(
        &self,
        address: FieldElement,
        method: &str,
        calldata: Vec<FieldElement>,
        nonce: Option<u64>,
    ) -> TransactionExecution {
        let calls = vec![Call { to: address, selector: get_selector_from_name(method).unwrap(), calldata }];

        let max_fee = FieldElement::from_hex_be(MAX_FEE_OVERRIDE).unwrap();

        match nonce {
            Some(nonce) => self.execute(calls).max_fee(max_fee).nonce(nonce.into()),
            None => self.execute(calls).max_fee(max_fee),
        }
    }

    fn declare_contract_params_sierra(
        &self,
        path_to_sierra: &str,
        path_to_casm: &str,
    ) -> (FieldElement, FlattenedSierraClass) {
        let sierra: SierraClass = serde_json::from_reader(
            std::fs::File::open(env!("CARGO_MANIFEST_DIR").to_owned() + "/" + path_to_sierra).unwrap(),
        )
        .unwrap();

        let flattened_class = sierra.flatten().unwrap();

        let casm: CompiledClass = serde_json::from_reader(
            std::fs::File::open(env!("CARGO_MANIFEST_DIR").to_owned() + "/" + path_to_casm).unwrap(),
        )
        .unwrap();

        (casm.class_hash().unwrap(), flattened_class)
    }

    fn declare_contract_params_legacy(&self, path_to_compiled_contract: &str) -> LegacyContractClass {
        let contract_artifact: LegacyContractClass = serde_json::from_reader(
            std::fs::File::open(env!("CARGO_MANIFEST_DIR").to_owned() + "/" + path_to_compiled_contract).unwrap(),
        )
        .unwrap();

        contract_artifact
    }
}

pub async fn assert_poll<F, Fut>(f: F, polling_time_ms: u64, max_poll_count: u32)
where
    F: Fn() -> Fut,
    Fut: Future<Output = bool>,
{
    for _poll_count in 0..max_poll_count {
        if f().await {
            return; // The provided function returned true, exit safely.
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(polling_time_ms)).await;
    }

    panic!("Max poll count exceeded.");
}

type TransactionReceiptResult = Result<MaybePendingTransactionReceipt, ProviderError>;

pub async fn get_transaction_receipt(
    rpc: &JsonRpcClient<HttpTransport>,
    transaction_hash: FieldElement,
) -> TransactionReceiptResult {
    // there is a delay between the transaction being available at the client
    // and the sealing of the block, hence sleeping for 100ms
    assert_poll(|| async { rpc.get_transaction_receipt(transaction_hash).await.is_ok() }, 100, 20).await;

    rpc.get_transaction_receipt(transaction_hash).await
}

pub async fn get_contract_address_from_deploy_tx(
    rpc: &JsonRpcClient<HttpTransport>,
    tx: &InvokeTransactionResult,
) -> Result<FieldElement, ProviderError> {
    let deploy_tx_hash = tx.transaction_hash;

    wait_for_transaction(rpc, deploy_tx_hash, "get_contract_address_from_deploy_tx").await.unwrap();

    let deploy_tx_receipt = get_transaction_receipt(rpc, deploy_tx_hash).await?;

    let contract_address = assert_matches!(
        deploy_tx_receipt,
        MaybePendingTransactionReceipt::Receipt(TransactionReceipt::Invoke(receipt)) => {
            receipt.events.iter().find(|e| e.keys[0] == get_selector_from_name("ContractDeployed").unwrap()).unwrap().data[0]
        }
    );
    Ok(contract_address)
}

pub async fn calculate_deployed_address(
    salt: ContractAddressSalt,
    class_hash: ClassHash,
    calldata: &Calldata,
    deployer_address: ContractAddress,
) -> FieldElement {
    let address = calculate_contract_address(salt, class_hash, calldata, deployer_address).unwrap();
    let bytes = address.0.0.0;
    FieldElement::from_bytes_be(&bytes).unwrap()
}
