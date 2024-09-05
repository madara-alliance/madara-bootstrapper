use std::time::Duration;

use anyhow::Context;
use starknet_ff::FieldElement;
use tokio::time::sleep;

use crate::contract_clients::utils::{declare_contract, DeclarationInput, RpcAccount};
use crate::utils::constants::{ARGENT_ACCOUNT_CASM_PATH, ARGENT_ACCOUNT_SIERRA_PATH};
use crate::utils::{save_to_json, JsonValueType};

pub struct ArgentSetup<'a> {
    account: RpcAccount<'a>,
}

pub struct ArgentSetupOutput {
    pub argent_class_hash: FieldElement,
}

impl<'a> ArgentSetup<'a> {
    pub fn new(account: RpcAccount<'a>) -> Self {
        Self { account }
    }

    pub async fn setup(&self) -> anyhow::Result<ArgentSetupOutput> {
        let argent_class_hash = declare_contract(DeclarationInput::DeclarationInputs(
            String::from(ARGENT_ACCOUNT_SIERRA_PATH),
            String::from(ARGENT_ACCOUNT_CASM_PATH),
            self.account.clone(),
        ))
        .await
        .context("Declaring argent class")?;
        log::debug!("📣 Argent Hash Declared");
        save_to_json("argent_class_hash", &JsonValueType::StringType(argent_class_hash.to_string()))
            .context("Saving argent class hash to json")?;
        sleep(Duration::from_secs(10)).await;

        Ok(ArgentSetupOutput { argent_class_hash })
    }
}
