use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use ethers::addressbook::Address;
use starknet_accounts::{Account, Execution};
use starknet_contract::ContractFactory;
use starknet_ff::FieldElement;
use starknet_providers::jsonrpc::HttpTransport;
use starknet_providers::JsonRpcClient;
use crate::bridge_deploy_utils::lib::fixtures::ThreadSafeMadaraClient;
use crate::bridge_deploy_utils::lib::utils::{build_single_owner_account, AccountActions};
use crate::bridge_deploy_utils::lib::Transaction;
use tokio::time::sleep;

const ERC20_SIERRA_PATH: &str = "src/contracts/erc20.sierra.json";
const ERC20_CASM_PATH: &str = "src/contracts/erc20.casm.json";

pub async fn deploy_eth_token_on_l2(rpc_provider_l2: &JsonRpcClient<HttpTransport>, minter: FieldElement, private_key: &str, address: &str) -> FieldElement {
    let account = build_single_owner_account(&rpc_provider_l2, private_key, address, false);

    let (class_hash, contract_artifact) = account.declare_contract_params_sierra(ERC20_SIERRA_PATH, ERC20_CASM_PATH);
    let flattened_class = contract_artifact.flatten().unwrap();

    account.declare(Arc::new(flattened_class), class_hash).send().await.expect("Unable to declare ERC20 token on L2");
    let contract_factory = ContractFactory::new(class_hash, account.clone());

    let deploy_tx = &contract_factory.deploy(
        vec![
            FieldElement::from_byte_slice_be("ether".as_bytes()).unwrap(), // Name
            FieldElement::from_byte_slice_be("ETH".as_bytes()).unwrap(),   // Symbol
            FieldElement::from_str("18").unwrap(),                         // Decimals
            FieldElement::from_str("10000000000000000000").unwrap(),       // Initial supply low
            FieldElement::from_str("0").unwrap(),                          // Initial supply high
            account.address(),                                             // recipient
            minter,                                                        // permitted_minter
            account.address(),                                             // provisional_governance_admin
            FieldElement::from_str("0").unwrap(),                          // upgrade_delay
        ],
        FieldElement::ZERO,
        true,
    ).send().await.expect("Unable to deploy ERC20 token on L2");

    deploy_tx.deployed_address()
}

pub async fn invoke_contract(
    rpc_provider: &JsonRpcClient<HttpTransport>,
    contract: FieldElement,
    method: &str,
    calldata: Vec<FieldElement>,
    priv_key: &str,
    address: &str
) {
    let account = build_single_owner_account(&rpc_provider, priv_key, address, false);

    account.invoke_contract(contract, method, calldata, None).send().await.expect("Error in invoking the contract !!");
}

pub async fn catch_and_execute_l1_messages(madara: &ThreadSafeMadaraClient) {
    // Wait for worker to catch L1 messages
    sleep(Duration::from_millis(20000)).await;
    let mut madara_write_lock = madara.write().await;
    madara_write_lock.create_block_with_pending_txs().await.expect("Failed to execute L1 Messages");
    println!("    <<>> l1 messages executed on l2 ✅");
}

pub fn pad_bytes(address: Address) -> Vec<u8> {
    let address_bytes = address.as_bytes();
    let mut padded_address_bytes = Vec::with_capacity(32);
    padded_address_bytes.extend(vec![0u8; 32 - address_bytes.len()]);
    padded_address_bytes.extend_from_slice(address_bytes);
    padded_address_bytes
}
