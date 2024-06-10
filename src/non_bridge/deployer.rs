use std::time::Duration;

use starknet_accounts::ConnectedAccount;
use starknet_ff::FieldElement;
use tokio::time::sleep;

use crate::bridge::helpers::account_actions::{get_contract_address_from_deploy_tx, AccountActions};
use crate::contract_clients::config::Config;
use crate::contract_clients::init_state::{declare_contract_middleware, DeclarationInput};
use crate::contract_clients::utils::build_single_owner_account;
use crate::utils::constants::{
    ARGENT_ACCOUNT_CASM_PATH, ARGENT_ACCOUNT_SIERRA_PATH, BRAAVOS_ACCOUNT_CASM_PATH, BRAAVOS_ACCOUNT_SIERRA_PATH,
    UDC_PATH,
};
use crate::utils::{convert_to_hex, save_to_json, wait_for_transaction, JsonValueType};
use crate::CliArgs;

pub async fn deploy_non_bridge_contracts(clients: &Config, arg_config: &CliArgs, account_address: FieldElement) {
    let udc_class_hash = declare_contract_middleware(DeclarationInput::LegacyDeclarationInputs(
        String::from(UDC_PATH),
        arg_config.rollup_seq_url.clone(),
    ))
    .await;
    log::debug!("UDC Class Hash Declared !!!");
    save_to_json("udc_class_hash", &JsonValueType::StringType(udc_class_hash.to_string())).unwrap();
    sleep(Duration::from_secs(10)).await;

    let user_account = build_single_owner_account(
        clients.provider_l2(),
        &arg_config.rollup_priv_key,
        &convert_to_hex(&account_address.to_string()),
        false,
    );

    let txn = user_account
        .invoke_contract(
            account_address,
            "deploy_contract",
            Vec::from([udc_class_hash, FieldElement::ZERO, FieldElement::ONE, FieldElement::ZERO]),
            None,
        )
        .send()
        .await
        .unwrap();
    wait_for_transaction(
        user_account.provider(),
        txn.transaction_hash,
        "deploy_non_bridge_contracts : deploy_contract : udc",
    )
    .await
    .unwrap();
    let udc_address = get_contract_address_from_deploy_tx(user_account.provider(), &txn).await.unwrap();
    save_to_json("udc_address", &JsonValueType::StringType(udc_address.to_string())).unwrap();
    log::debug!("udc_address : {:?}", udc_address);

    let argent_class_hash = declare_contract_middleware(DeclarationInput::DeclarationInputs(
        String::from(ARGENT_ACCOUNT_SIERRA_PATH),
        String::from(ARGENT_ACCOUNT_CASM_PATH),
        user_account.clone(),
    ))
    .await;
    log::debug!("Argent Hash Declared !!!");
    save_to_json("argent_class_hash", &JsonValueType::StringType(argent_class_hash.to_string())).unwrap();
    sleep(Duration::from_secs(10)).await;

    let braavos_class_hash = declare_contract_middleware(DeclarationInput::DeclarationInputs(
        String::from(BRAAVOS_ACCOUNT_SIERRA_PATH),
        String::from(BRAAVOS_ACCOUNT_CASM_PATH),
        user_account.clone(),
    ))
    .await;
    log::debug!("Braavos Hash Declared !!!");
    save_to_json("braavos_class_hash", &JsonValueType::StringType(braavos_class_hash.to_string())).unwrap();
    sleep(Duration::from_secs(10)).await;
}
