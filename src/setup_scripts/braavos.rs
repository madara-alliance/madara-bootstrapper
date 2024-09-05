use std::time::Duration;

use anyhow::Context;
use starknet_accounts::{Account, ConnectedAccount};
use starknet_ff::FieldElement;
use tokio::time::sleep;

use crate::contract_clients::utils::{declare_contract, DeclarationInput, RpcAccount};
use crate::helpers::account_actions::{get_contract_address_from_deploy_tx, AccountActions};
use crate::utils::constants::{
    BRAAVOS_ACCOUNT_CASM_PATH, BRAAVOS_ACCOUNT_SIERRA_PATH, BRAAVOS_AGGREGATOR_PATH, BRAAVOS_BASE_ACCOUNT_CASM_PATH,
    BRAAVOS_BASE_ACCOUNT_SIERRA_PATH,
};
use crate::utils::{save_to_json, wait_for_transaction, JsonValueType};
use crate::CliArgs;

pub struct BraavosSetup<'a> {
    account: RpcAccount<'a>,
    arg_config: &'a CliArgs,
}

pub struct BraavosSetupOutput {
    pub braavos_class_hash: FieldElement,
}

impl<'a> BraavosSetup<'a> {
    pub fn new(account: RpcAccount<'a>, arg_config: &'a CliArgs) -> Self {
        Self { account, arg_config }
    }

    pub async fn setup(&self) -> anyhow::Result<BraavosSetupOutput> {
        let braavos_class_hash = declare_contract(DeclarationInput::DeclarationInputs(
            String::from(BRAAVOS_ACCOUNT_SIERRA_PATH),
            String::from(BRAAVOS_ACCOUNT_CASM_PATH),
            self.account.clone(),
        ))
        .await
        .context("Declaring braavos class")?;
        log::debug!("📣 Braavos Account class hash declared.");
        save_to_json("braavos_class_hash", &JsonValueType::StringType(braavos_class_hash.to_string()))
            .context("Saving braavos class hash to json")?;
        sleep(Duration::from_secs(10)).await;

        let braavos_base_account_class_hash = declare_contract(DeclarationInput::DeclarationInputs(
            String::from(BRAAVOS_BASE_ACCOUNT_SIERRA_PATH),
            String::from(BRAAVOS_BASE_ACCOUNT_CASM_PATH),
            self.account.clone(),
        ))
        .await
        .context("Declaring braavos base account class")?;
        log::debug!("📣 Braavos Base Account class hash declared.");
        save_to_json(
            "braavos_base_account_class_hash",
            &JsonValueType::StringType(braavos_base_account_class_hash.to_string()),
        )
        .context("Saving braavos base account class hash to json")?;
        sleep(Duration::from_secs(10)).await;

        let braavos_aggregator_class_hash = declare_contract(DeclarationInput::LegacyDeclarationInputs(
            String::from(BRAAVOS_AGGREGATOR_PATH),
            self.arg_config.rollup_seq_url.clone(),
        ))
        .await
        .context("Declaring braavos aggregator class")?;
        log::debug!("📣 Braavos Aggregator class hash declared.");
        save_to_json(
            "braavos_aggregator_class_hash",
            &JsonValueType::StringType(braavos_aggregator_class_hash.to_string()),
        )
        .context("Saving braavos aggregator class hash to json")?;
        sleep(Duration::from_secs(10)).await;

        let deploy_tx = self
            .account
            .invoke_contract(
                self.account.address(),
                "deploy_contract",
                vec![braavos_aggregator_class_hash, FieldElement::ZERO, FieldElement::ZERO, FieldElement::ZERO],
                None,
            )
            .context("Making braavos deploy_contract transaction")?
            .send()
            .await
            .context("Error deploying the contract proxy")?;
        wait_for_transaction(self.account.provider(), deploy_tx.transaction_hash, "deploy_eth_token_on_l2 : deploy")
            .await
            .context("Waiting for braavos deploy_contract transaction to settle")?;
        let contract_address = get_contract_address_from_deploy_tx(self.account.provider(), &deploy_tx)
            .await
            .context("Getting resulting braavos contract address")?;

        log::info!("*️⃣ Braavos Aggregator deployed. [Braavos Aggregator : {:?}]", contract_address);

        Ok(BraavosSetupOutput { braavos_class_hash })
    }
}
