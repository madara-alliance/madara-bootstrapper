use std::time::Duration;

use starknet_ff::FieldElement;
use tokio::time::sleep;

use crate::contract_clients::init_state::{declare_contract_util_func, DeclarationInput};
use crate::contract_clients::utils::RpcAccount;
use crate::utils::constants::{ERC20_CASM_PATH, ERC20_SIERRA_PATH};
use crate::utils::{save_to_json, JsonValueType};

pub struct Erc20BridgeSetupOutput {
    pub erc20_cairo_one_class_hash: FieldElement,
}

pub async fn erc20_bridge_init_func(user_account: RpcAccount<'_>) -> Erc20BridgeSetupOutput {
    let erc20_cairo_one_class_hash = declare_contract_util_func(DeclarationInput::DeclarationInputs(
        String::from(ERC20_SIERRA_PATH),
        String::from(ERC20_CASM_PATH),
        user_account.clone(),
    ))
    .await;
    log::debug!("ERC20 Class Hash declared !!! : {:?}", erc20_cairo_one_class_hash);
    save_to_json("erc20_cairo_one_class_hash", &JsonValueType::StringType(erc20_cairo_one_class_hash.to_string()))
        .unwrap();
    sleep(Duration::from_secs(10)).await;

    Erc20BridgeSetupOutput { erc20_cairo_one_class_hash }
}
