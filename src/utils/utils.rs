use ethers::addressbook::Address;
use starknet_ff::FieldElement;
use starknet_providers::jsonrpc::HttpTransport;
use starknet_providers::JsonRpcClient;
use crate::bridge::helpers::account_actions::AccountActions;
use crate::bridge::helpers::deploy_utils::build_single_owner_account;

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

pub fn pad_bytes(address: Address) -> Vec<u8> {
    let address_bytes = address.as_bytes();
    let mut padded_address_bytes = Vec::with_capacity(32);
    padded_address_bytes.extend(vec![0u8; 32 - address_bytes.len()]);
    padded_address_bytes.extend_from_slice(address_bytes);
    padded_address_bytes
}
