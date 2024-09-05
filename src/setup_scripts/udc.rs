use std::time::Duration;

use anyhow::Context;
use starknet_accounts::ConnectedAccount;
use starknet_ff::FieldElement;
use tokio::time::sleep;

use crate::contract_clients::utils::{declare_contract, DeclarationInput, RpcAccount};
use crate::helpers::account_actions::{get_contract_address_from_deploy_tx, AccountActions};
use crate::utils::constants::UDC_PATH;
use crate::utils::{save_to_json, wait_for_transaction, JsonValueType};
use crate::CliArgs;

pub struct UdcSetup<'a> {
    account: RpcAccount<'a>,
    account_address: FieldElement,
    arg_config: &'a CliArgs,
}

pub struct UdcSetupOutput {
    pub udc_class_hash: FieldElement,
    pub udc_address: FieldElement,
}

impl<'a> UdcSetup<'a> {
    pub fn new(account: RpcAccount<'a>, account_address: FieldElement, arg_config: &'a CliArgs) -> Self {
        Self { account, account_address, arg_config }
    }

    pub async fn setup(&self) -> anyhow::Result<UdcSetupOutput> {
        let udc_class_hash = declare_contract(DeclarationInput::LegacyDeclarationInputs(
            String::from(UDC_PATH),
            self.arg_config.rollup_seq_url.clone(),
        ))
        .await
        .context("Declaring UDC class")?;
        log::debug!("📣 UDC Class Hash Declared.");
        save_to_json("udc_class_hash", &JsonValueType::StringType(udc_class_hash.to_string()))
            .context("Saving udc class hash to json")?;
        sleep(Duration::from_secs(10)).await;

        let txn = self
            .account
            .invoke_contract(
                self.account_address,
                "deploy_contract",
                Vec::from([udc_class_hash, FieldElement::ZERO, FieldElement::ONE, FieldElement::ZERO]),
                None,
            )
            .context("Making deploy_contract transaction")?
            .send()
            .await
            .context("Sending deploy_contract transaction")?;
        wait_for_transaction(
            self.account.provider(),
            txn.transaction_hash,
            "deploy_non_bridge_contracts : deploy_contract : udc",
        )
        .await
        .context("Waiting for deploy_contract transaction to settle")?;
        let udc_address = get_contract_address_from_deploy_tx(self.account.provider(), &txn)
            .await
            .context("Getting resulting contract address")?;
        save_to_json("udc_address", &JsonValueType::StringType(udc_address.to_string()))
            .context("Saving UDC address to json")?;
        log::debug!("📣 udc_address : {:?}", udc_address);

        Ok(UdcSetupOutput { udc_class_hash, udc_address })
    }
}
