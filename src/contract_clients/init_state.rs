use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

use cairo_vm::serde::deserialize_program::{Attribute, BuiltinName, HintParams, Identifier, ReferenceManager};
use cairo_vm::types::program::Program;
use cairo_vm::types::relocatable::MaybeRelocatable;
use clap::arg;
use ethers::core::rand;
use rand::Rng;
use scale_info::Field;
use starknet_accounts::{Account, AccountFactory, ConnectedAccount, OpenZeppelinAccountFactory};
use starknet_api::hash::StarkHash;
use starknet_api::transaction::TransactionHash;
use starknet_core::types::contract::legacy::LegacyContractClass;
use starknet_core::types::contract::{CompiledClass, SierraClass};
use starknet_core::types::{BlockId, BlockTag};
use starknet_ff::FieldElement;
use starknet_providers::jsonrpc::HttpTransport;
use starknet_providers::{JsonRpcClient, Provider};
use starknet_signers::{LocalWallet, SigningKey};
use subxt::ext::futures::future::err;

use crate::bridge::helpers::account_actions::{get_contract_address_from_deploy_tx, AccountActions};
use crate::contract_clients::config::Config;
use crate::contract_clients::subxt_funcs::appchain::runtime_types::blockifier::execution::contract_class::{
    ContractClassV0, ContractClassV0Inner, ContractClassV1,
};
use crate::contract_clients::subxt_funcs::appchain::runtime_types::cairo_vm::serde::deserialize_program::BuiltinName as BuiltinNameSubxt;
use crate::contract_clients::subxt_funcs::appchain::runtime_types::cairo_vm::types::program::{
    HintsCollection, Program as ProgramSubxt, SharedProgramData,
};
use crate::contract_clients::subxt_funcs::appchain::runtime_types::starknet_api::deprecated_contract_class::{
    EntryPoint, EntryPointType,
};
use crate::contract_clients::subxt_funcs::{declare_contract_subxt, declare_transaction_build_subxt, toggle_fee};
use crate::contract_clients::utils::{build_single_owner_account, RpcAccount};
use crate::utils::constants::{
    ERC20_CASM_PATH, ERC20_SIERRA_PATH, LEGACY_BRIDGE_PATH, LEGACY_BRIDGE_PROGRAM_PATH, OZ_ACCOUNT_CASM_PATH,
    OZ_ACCOUNT_PATH, OZ_ACCOUNT_PROGRAM_PATH, OZ_ACCOUNT_SIERRA_PATH, PROXY_PATH, PROXY_PROGRAM_PATH,
};
use crate::utils::mapper::{
    map_builtins, map_constants, map_data, map_entrypoint_selector, map_error_message_attributes, map_hints,
    map_hints_ranges, map_identifiers, map_instruction_locations, map_main, map_program_end, map_program_start,
    map_reference_manager,
};
use crate::utils::{invoke_contract, wait_for_transaction};
use crate::{contract_clients, CliArgs};

pub async fn init_and_deploy_eth_and_account(
    clients: &Config,
    arg_config: &CliArgs,
) -> (FieldElement, FieldElement, FieldElement, FieldElement, FieldElement, FieldElement, FieldElement) {
    toggle_fee(true).await.expect("Error in disabling the fee on configured app-chain");

    let legacy_eth_bridge_class_hash = declare_contract_using_subxt(DeclarationInput::LegacyDeclarationInputs(
        String::from(LEGACY_BRIDGE_PATH),
        String::from(LEGACY_BRIDGE_PROGRAM_PATH),
    ))
    .await;
    let oz_account_class_hash = declare_contract_using_subxt(DeclarationInput::LegacyDeclarationInputs(
        String::from(OZ_ACCOUNT_PATH),
        String::from(OZ_ACCOUNT_PROGRAM_PATH),
    ))
    .await;
    let proxy_class_hash = declare_contract_using_subxt(DeclarationInput::LegacyDeclarationInputs(
        String::from(PROXY_PATH),
        String::from(PROXY_PROGRAM_PATH),
    ))
    .await;

    let account_address =
        deploy_account_using_priv_key(arg_config.rollup_priv_key.clone(), clients.provider_l2(), oz_account_class_hash)
            .await;
    let user_account = build_single_owner_account(
        clients.provider_l2(),
        &*arg_config.rollup_priv_key,
        &account_address.to_string(),
        false,
    );
    // cairo 1 declarations through account
    let erc_20_class_hash = declare_contract_using_subxt(DeclarationInput::DeclarationInputs(
        String::from(ERC20_SIERRA_PATH),
        String::from(ERC20_CASM_PATH),
        user_account.clone(),
    ))
    .await;

    let eth_proxy_address = deploy_proxy_contract(
        &user_account,
        account_address,
        proxy_class_hash,
        // salt taken from : https://sepolia.starkscan.co/tx/0x06a5a493cf33919e58aa4c75777bffdef97c0e39cac968896d7bee8cc67905a1
        FieldElement::from_str("0x322c2610264639f6b2cee681ac53fa65c37e187ea24292d1b21d859c55e1a78").unwrap(),
        FieldElement::ONE,
    )
    .await;
    let eth_bridge_proxy_address = deploy_proxy_contract(
        &user_account,
        account_address,
        proxy_class_hash,
        FieldElement::from_str("0xkarnot").unwrap(),
        FieldElement::ZERO,
    )
    .await;

    init_governance_proxy(&user_account, eth_proxy_address, &arg_config.rollup_priv_key).await;
    init_governance_proxy(&user_account, eth_bridge_proxy_address, &arg_config.rollup_priv_key).await;

    let token_bridge_proxy_address = deploy_proxy_contract(
        &user_account,
        account_address,
        proxy_class_hash,
        FieldElement::from_str("0xkarnot").unwrap(),
        FieldElement::ZERO,
    )
    .await;

    (
        erc_20_class_hash,
        legacy_eth_bridge_class_hash,
        account_address,
        eth_proxy_address,
        eth_bridge_proxy_address,
        token_bridge_proxy_address,
        proxy_class_hash,
    )
}

enum DeclarationInput {
    // inputs : sierra_path, casm_path
    DeclarationInputs(String, String, RpcAccount<'static>),
    // input : artifact_path
    LegacyDeclarationInputs(String, String),
}

async fn declare_contract_using_subxt(input: DeclarationInput) -> FieldElement {
    match input {
        DeclarationInput::DeclarationInputs(sierra_path, casm_path, account) => {
            let (class_hash, sierra) = account.declare_contract_params_sierra(&sierra_path, &casm_path);
            account
                .declare(Arc::new(sierra.clone().flatten().unwrap()), class_hash)
                .send()
                .await
                .expect("Error in declaring the contract using Cairo 1 declaration using the provided account !!!");
            sierra.class_hash().unwrap()
        }
        DeclarationInput::LegacyDeclarationInputs(artifact_path, program_artifact_path) => {
            let contract_artifact: LegacyContractClass = serde_json::from_reader(
                std::fs::File::open(env!("CARGO_MANIFEST_DIR").to_owned() + "/" + &artifact_path).unwrap(),
            )
            .unwrap();

            let entrypoints = contract_artifact.entry_points_by_type.clone();

            let p = Program::from_file(
                (env!("CARGO_MANIFEST_DIR").to_owned() + "/" + &program_artifact_path).as_ref(),
                None,
            )
            .unwrap();

            let program: ProgramSubxt = ProgramSubxt {
                shared_program_data: SharedProgramData {
                    data: map_data(&p),
                    hints_collection: HintsCollection { hints: map_hints(&p), hints_ranges: map_hints_ranges(&p) },
                    main: map_main(&p),
                    start: map_program_start(&p),
                    end: map_program_end(&p),
                    error_message_attributes: map_error_message_attributes(&p),
                    instruction_locations: map_instruction_locations(&p),
                    identifiers: map_identifiers(&p),
                    reference_manager: map_reference_manager(&p),
                },
                constatnts: map_constants(&p),
                builtins: map_builtins(&p),
            };

            declare_transaction_build_subxt(
                contract_artifact.class_hash().unwrap(),
                FieldElement::ONE,
                contract_artifact.clone(),
                program,
                map_entrypoint_selector(entrypoints),
            )
            .await;

            contract_artifact.class_hash().unwrap()
        }
    }
}

async fn deploy_account_using_priv_key(
    priv_key: String,
    provider: &JsonRpcClient<HttpTransport>,
    oz_account_class_hash: FieldElement,
) -> FieldElement {
    let chain_id = provider.chain_id().await.unwrap();
    let signer = Arc::new(LocalWallet::from_signing_key(SigningKey::from_secret_scalar(
        FieldElement::from_hex_be(&priv_key).unwrap(),
    )));
    let mut oz_account_factory =
        OpenZeppelinAccountFactory::new(oz_account_class_hash, chain_id, signer, provider).await.unwrap();
    oz_account_factory.set_block_id(BlockId::Tag(BlockTag::Pending));

    let deploy_txn = oz_account_factory.deploy(FieldElement::ZERO);
    let account_address = deploy_txn.address();
    log::debug!("OZ Account Deployed : {:?}", account_address);

    let sent_txn = deploy_txn.send().await.expect("Error in deploying the OZ account");
    wait_for_transaction(provider, sent_txn.transaction_hash).await.unwrap();

    account_address
}

async fn deploy_proxy_contract(
    account: &RpcAccount<'_>,
    account_address: FieldElement,
    class_hash: FieldElement,
    salt: FieldElement,
    deploy_from_zero: FieldElement,
) -> FieldElement {
    let mut rng = rand::thread_rng();
    let random: u32 = rng.gen();

    let txn = account
        .invoke_contract(
            account_address,
            "deploy_contract",
            vec![class_hash, salt, FieldElement::ONE, FieldElement::ZERO, deploy_from_zero],
            None,
        )
        .send()
        .await
        .expect("Error deploying the contract proxy.");

    wait_for_transaction(account.provider(), txn.transaction_hash).await.unwrap();
    get_contract_address_from_deploy_tx(account.provider(), &txn).await.unwrap()
}

async fn init_governance_proxy(account: &RpcAccount<'_>, contract_address: FieldElement, p_key: &str) {
    let txn = invoke_contract(
        account.provider(),
        contract_address,
        "init_governance",
        vec![],
        p_key,
        &account.address().to_string(),
    )
    .await;
    wait_for_transaction(account.provider(), txn.transaction_hash).await.unwrap();
}
