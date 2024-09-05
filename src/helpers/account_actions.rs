use std::future::Future;
use std::path::Path;

use anyhow::{bail, Context};
use assert_matches::assert_matches;
use async_trait::async_trait;
use serde::de::DeserializeOwned;
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
use starknet_providers::{JsonRpcClient, Provider};
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
    ) -> anyhow::Result<TransactionExecution>;

    fn declare_contract_params_sierra(
        &self,
        path_to_sierra: &str,
        path_to_casm: &str,
    ) -> anyhow::Result<(FieldElement, FlattenedSierraClass)>;
    fn declare_contract_params_legacy(&self, path_to_compiled_contract: &str) -> anyhow::Result<LegacyContractClass>;
}

impl AccountActions for SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet> {
    fn invoke_contract(
        &self,
        address: FieldElement,
        method: &str,
        calldata: Vec<FieldElement>,
        nonce: Option<u64>,
    ) -> anyhow::Result<TransactionExecution> {
        let calls = vec![Call {
            to: address,
            selector: get_selector_from_name(method)
                .with_context(|| format!("Getting function selector for method {method}"))?,
            calldata,
        }];

        let max_fee = FieldElement::from_hex_be(MAX_FEE_OVERRIDE).expect("Converting a constant to field element");

        Ok(match nonce {
            Some(nonce) => self.execute(calls).max_fee(max_fee).nonce(nonce.into()),
            None => self.execute(calls).max_fee(max_fee),
        })
    }

    fn declare_contract_params_sierra(
        &self,
        path_to_sierra: &str,
        path_to_casm: &str,
    ) -> anyhow::Result<(FieldElement, FlattenedSierraClass)> {
        let path = format!("{}/{}", env!("CARGO_MANIFEST_DIR").to_owned(), path_to_sierra);
        let sierra: SierraClass =
            deser_json_artifact(&path).with_context(|| format!("Loading Sierra artifact at path {}", path))?;

        let flattened_class =
            sierra.flatten().with_context(|| format!("Flattening sierra class loaded from path {}", path_to_sierra))?;

        let path = format!("{}/{}", env!("CARGO_MANIFEST_DIR").to_owned(), path_to_casm);
        let casm: CompiledClass =
            deser_json_artifact(&path).with_context(|| format!("Loading compiled CASM artifact at path {}", path))?;

        Ok((
            casm.class_hash().with_context(|| format!("Getting class hash from casm artifact {}", path_to_casm))?,
            flattened_class,
        ))
    }

    fn declare_contract_params_legacy(&self, path_to_compiled_contract: &str) -> anyhow::Result<LegacyContractClass> {
        let path = format!("{}/{}", env!("CARGO_MANIFEST_DIR").to_owned(), path_to_compiled_contract);
        let contract_artifact: LegacyContractClass =
            deser_json_artifact(&path).with_context(|| format!("Loading legacy CASM artifact at path {}", path))?;
        Ok(contract_artifact)
    }
}

fn deser_json_artifact<T: DeserializeOwned>(path: impl AsRef<Path>) -> anyhow::Result<T> {
    let res = serde_json::from_reader(std::fs::File::open(&path).context("Opening file")?)
        .context("Reading and deserializing file")?;
    Ok(res)
}

pub async fn assert_poll<F, Fut>(f: F, polling_time_ms: u64, max_poll_count: u32) -> anyhow::Result<()>
where
    F: Fn() -> Fut,
    Fut: Future<Output = bool>,
{
    for _poll_count in 0..max_poll_count {
        if f().await {
            return Ok(()); // The provided function returned true, exit safely.
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(polling_time_ms)).await;
    }

    bail!("Max poll count exceeded");
}

pub async fn get_transaction_receipt(
    rpc: &JsonRpcClient<HttpTransport>,
    transaction_hash: FieldElement,
) -> anyhow::Result<MaybePendingTransactionReceipt> {
    // there is a delay between the transaction being available at the client
    // and the sealing of the block, hence sleeping for 100ms
    assert_poll(|| async { rpc.get_transaction_receipt(transaction_hash).await.is_ok() }, 100, 20)
        .await
        .with_context(|| format!("Getting transaction receipt for transaction hash {:#x}", transaction_hash))?;

    rpc.get_transaction_receipt(transaction_hash)
        .await
        .with_context(|| format!("Getting transaction receipt for transaction hash {:#x}", transaction_hash))
}

pub async fn get_contract_address_from_deploy_tx(
    rpc: &JsonRpcClient<HttpTransport>,
    tx: &InvokeTransactionResult,
) -> anyhow::Result<FieldElement> {
    let deploy_tx_hash = tx.transaction_hash;

    wait_for_transaction(rpc, deploy_tx_hash, "get_contract_address_from_deploy_tx").await?;

    let deploy_tx_receipt = get_transaction_receipt(rpc, deploy_tx_hash).await?;

    let contract_address = assert_matches!(
        deploy_tx_receipt,
        MaybePendingTransactionReceipt::Receipt(TransactionReceipt::Invoke(receipt)) => {
            receipt.events.iter().find(|e| e.keys[0] == get_selector_from_name("ContractDeployed").expect("Converting constant to function selector")).context("The RPC did not return any contract deployed event")?.data[0]
        }
    );
    Ok(contract_address)
}

pub async fn calculate_deployed_address(
    salt: ContractAddressSalt,
    class_hash: ClassHash,
    calldata: &Calldata,
    deployer_address: ContractAddress,
) -> anyhow::Result<FieldElement> {
    let address = calculate_contract_address(salt, class_hash, calldata, deployer_address)
        .context("Calculating deployed contract address")?;
    let bytes = address.0.0.0;
    FieldElement::from_bytes_be(&bytes).context("Converting contract address to field element")
}
