use std::time::Duration;

use starknet_ff::FieldElement;
use tokio::time::sleep;

use crate::contract_clients::utils::{declare_contract_util_func, DeclarationInput, RpcAccount};
use crate::utils::constants::{BRAAVOS_ACCOUNT_CASM_PATH, BRAAVOS_ACCOUNT_SIERRA_PATH};
use crate::utils::{save_to_json, JsonValueType};

pub struct BraavosSetup<'a> {
    account: RpcAccount<'a>,
}

pub struct BraavosSetupOutput {
    pub braavos_class_hash: FieldElement,
}

impl<'a> BraavosSetup<'a> {
    pub fn new(account: RpcAccount<'a>) -> Self {
        Self { account }
    }

    pub async fn setup(&self) -> BraavosSetupOutput {
        let braavos_class_hash = declare_contract_util_func(DeclarationInput::DeclarationInputs(
            String::from(BRAAVOS_ACCOUNT_SIERRA_PATH),
            String::from(BRAAVOS_ACCOUNT_CASM_PATH),
            self.account.clone(),
        ))
        .await;
        log::debug!("Braavos Hash Declared !!!");
        save_to_json("braavos_class_hash", &JsonValueType::StringType(braavos_class_hash.to_string())).unwrap();
        sleep(Duration::from_secs(10)).await;

        BraavosSetupOutput { braavos_class_hash }
    }
}
