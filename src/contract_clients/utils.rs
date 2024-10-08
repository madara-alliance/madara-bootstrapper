use std::sync::Arc;

use ethers::types::U256;
use hex::encode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use starknet::accounts::{
    Account, AccountFactory, ConnectedAccount, ExecutionEncoding, OpenZeppelinAccountFactory, SingleOwnerAccount,
};
use starknet::core::types::contract::legacy::LegacyContractClass;
use starknet::core::types::{BlockId, BlockTag, DeclareTransactionResult, Felt, FunctionCall};
use starknet::core::utils::get_selector_from_name;
use starknet::providers::jsonrpc::{HttpTransport, JsonRpcClient};
use starknet::providers::Provider;
use starknet::signers::{LocalWallet, SigningKey};
use starknet_core::types::contract::{CompiledClass, SierraClass};
use starknet_core::types::BlockTag::Pending;
use starknet_types_core::hash::{Pedersen, StarkHash};

use crate::contract_clients::utils::DeclarationInput::{DeclarationInputs, LegacyDeclarationInputs};
use crate::helpers::account_actions::{get_contract_address_from_deploy_tx, AccountActions};
use crate::utils::{invoke_contract, save_to_json, wait_for_transaction, JsonValueType};
use crate::CliArgs;
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EntryPointsByType {
    pub constructor: Vec<SierraEntryPoint>,
    pub external: Vec<SierraEntryPoint>,
    pub l1_handler: Vec<SierraEntryPoint>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SierraEntryPoint {
    pub selector: Felt,
    pub function_idx: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CompressedLegacyContractClass {
    pub program: Vec<u8>,
    pub entry_points_by_type: LegacyEntryPointsByType,
    pub abi: Option<Vec<LegacyContractAbiEntry>>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LegacyEntryPointsByType {
    #[serde(rename = "CONSTRUCTOR")]
    pub constructor: Vec<LegacyContractEntryPoint>,
    #[serde(rename = "EXTERNAL")]
    pub external: Vec<LegacyContractEntryPoint>,
    #[serde(rename = "L1_HANDLER")]
    pub l1_handler: Vec<LegacyContractEntryPoint>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LegacyContractEntryPoint {
    pub offset: u64,
    pub selector: Felt,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LegacyContractAbiEntry {
    Function(LegacyFunctionAbiEntry),
    Event(LegacyEventAbiEntry),
    Struct(LegacyStructAbiEntry),
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LegacyFunctionAbiEntry {
    pub r#type: LegacyFunctionAbiType,
    pub name: String,
    pub inputs: Vec<LegacyTypedParameter>,
    pub outputs: Vec<LegacyTypedParameter>,
    #[serde(rename = "stateMutability")]
    pub state_mutability: Option<FunctionStateMutability>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LegacyEventAbiEntry {
    pub r#type: LegacyEventAbiType,
    pub name: String,
    pub keys: Vec<LegacyTypedParameter>,
    pub data: Vec<LegacyTypedParameter>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LegacyStructAbiEntry {
    pub r#type: LegacyStructAbiType,
    pub name: String,
    pub size: u64,
    pub members: Vec<LegacyStructMember>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LegacyStructMember {
    pub name: String,
    pub r#type: String,
    pub offset: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LegacyTypedParameter {
    pub name: String,
    pub r#type: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LegacyFunctionAbiType {
    #[serde(rename = "function")]
    Function,
    #[serde(rename = "l1_handler")]
    L1Handler,
    #[serde(rename = "constructor")]
    Constructor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LegacyEventAbiType {
    #[serde(rename = "event")]
    Event,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LegacyStructAbiType {
    #[serde(rename = "struct")]
    Struct,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FunctionStateMutability {
    #[serde(rename = "view")]
    View,
}

impl From<starknet_core::types::EntryPointsByType> for EntryPointsByType {
    fn from(entry_points_by_type: starknet_core::types::EntryPointsByType) -> Self {
        EntryPointsByType {
            constructor: entry_points_by_type
                .constructor
                .into_iter()
                .map(|sierra_entry_point| sierra_entry_point.into())
                .collect(),
            external: entry_points_by_type
                .external
                .into_iter()
                .map(|sierra_entry_point| sierra_entry_point.into())
                .collect(),
            l1_handler: entry_points_by_type
                .l1_handler
                .into_iter()
                .map(|sierra_entry_point| sierra_entry_point.into())
                .collect(),
        }
    }
}

impl From<EntryPointsByType> for starknet_core::types::EntryPointsByType {
    fn from(entry_points_by_type: EntryPointsByType) -> Self {
        starknet_core::types::EntryPointsByType {
            constructor: entry_points_by_type
                .constructor
                .into_iter()
                .map(|sierra_entry_point| sierra_entry_point.into())
                .collect(),
            external: entry_points_by_type
                .external
                .into_iter()
                .map(|sierra_entry_point| sierra_entry_point.into())
                .collect(),
            l1_handler: entry_points_by_type
                .l1_handler
                .into_iter()
                .map(|sierra_entry_point| sierra_entry_point.into())
                .collect(),
        }
    }
}

impl From<starknet_core::types::SierraEntryPoint> for SierraEntryPoint {
    fn from(sierra_entry_point: starknet_core::types::SierraEntryPoint) -> Self {
        SierraEntryPoint { selector: sierra_entry_point.selector, function_idx: sierra_entry_point.function_idx }
    }
}

impl From<SierraEntryPoint> for starknet_core::types::SierraEntryPoint {
    fn from(sierra_entry_point: SierraEntryPoint) -> Self {
        starknet_core::types::SierraEntryPoint {
            selector: sierra_entry_point.selector,
            function_idx: sierra_entry_point.function_idx,
        }
    }
}

impl From<starknet_core::types::CompressedLegacyContractClass> for CompressedLegacyContractClass {
    fn from(compressed_legacy_contract_class: starknet_core::types::CompressedLegacyContractClass) -> Self {
        CompressedLegacyContractClass {
            program: compressed_legacy_contract_class.program,
            entry_points_by_type: compressed_legacy_contract_class.entry_points_by_type.into(),
            abi: compressed_legacy_contract_class
                .abi
                .map(|abi| abi.into_iter().map(|legacy_contract_abi_entry| legacy_contract_abi_entry.into()).collect()),
        }
    }
}

impl From<CompressedLegacyContractClass> for starknet_core::types::CompressedLegacyContractClass {
    fn from(compressed_legacy_contract_class: CompressedLegacyContractClass) -> Self {
        starknet_core::types::CompressedLegacyContractClass {
            program: compressed_legacy_contract_class.program,
            entry_points_by_type: compressed_legacy_contract_class.entry_points_by_type.into(),
            abi: compressed_legacy_contract_class
                .abi
                .map(|abi| abi.into_iter().map(|legacy_contract_abi_entry| legacy_contract_abi_entry.into()).collect()),
        }
    }
}

impl From<starknet_core::types::LegacyEntryPointsByType> for LegacyEntryPointsByType {
    fn from(legacy_entry_points_by_type: starknet_core::types::LegacyEntryPointsByType) -> Self {
        LegacyEntryPointsByType {
            constructor: legacy_entry_points_by_type
                .constructor
                .into_iter()
                .map(|legacy_contract_entry_point| legacy_contract_entry_point.into())
                .collect(),
            external: legacy_entry_points_by_type
                .external
                .into_iter()
                .map(|legacy_contract_entry_point| legacy_contract_entry_point.into())
                .collect(),
            l1_handler: legacy_entry_points_by_type
                .l1_handler
                .into_iter()
                .map(|legacy_contract_entry_point| legacy_contract_entry_point.into())
                .collect(),
        }
    }
}

impl From<LegacyEntryPointsByType> for starknet_core::types::LegacyEntryPointsByType {
    fn from(legacy_entry_points_by_type: LegacyEntryPointsByType) -> Self {
        starknet_core::types::LegacyEntryPointsByType {
            constructor: legacy_entry_points_by_type
                .constructor
                .into_iter()
                .map(|legacy_contract_entry_point| legacy_contract_entry_point.into())
                .collect(),
            external: legacy_entry_points_by_type
                .external
                .into_iter()
                .map(|legacy_contract_entry_point| legacy_contract_entry_point.into())
                .collect(),
            l1_handler: legacy_entry_points_by_type
                .l1_handler
                .into_iter()
                .map(|legacy_contract_entry_point| legacy_contract_entry_point.into())
                .collect(),
        }
    }
}

impl From<starknet_core::types::LegacyContractEntryPoint> for LegacyContractEntryPoint {
    fn from(legacy_contract_entry_point: starknet_core::types::LegacyContractEntryPoint) -> Self {
        LegacyContractEntryPoint {
            offset: legacy_contract_entry_point.offset,
            selector: legacy_contract_entry_point.selector,
        }
    }
}

impl From<LegacyContractEntryPoint> for starknet_core::types::LegacyContractEntryPoint {
    fn from(legacy_contract_entry_point: LegacyContractEntryPoint) -> Self {
        starknet_core::types::LegacyContractEntryPoint {
            offset: legacy_contract_entry_point.offset,
            selector: legacy_contract_entry_point.selector,
        }
    }
}

impl From<starknet_core::types::LegacyContractAbiEntry> for LegacyContractAbiEntry {
    fn from(legacy_contract_abi_entry: starknet_core::types::LegacyContractAbiEntry) -> Self {
        match legacy_contract_abi_entry {
            starknet_core::types::LegacyContractAbiEntry::Function(legacy_function_abi_entry) => {
                LegacyContractAbiEntry::Function(legacy_function_abi_entry.into())
            }
            starknet_core::types::LegacyContractAbiEntry::Event(legacy_event_abi_entry) => {
                LegacyContractAbiEntry::Event(legacy_event_abi_entry.into())
            }
            starknet_core::types::LegacyContractAbiEntry::Struct(legacy_struct_abi_entry) => {
                LegacyContractAbiEntry::Struct(legacy_struct_abi_entry.into())
            }
        }
    }
}

impl From<LegacyContractAbiEntry> for starknet_core::types::LegacyContractAbiEntry {
    fn from(legacy_contract_abi_entry: LegacyContractAbiEntry) -> Self {
        match legacy_contract_abi_entry {
            LegacyContractAbiEntry::Function(legacy_function_abi_entry) => {
                starknet_core::types::LegacyContractAbiEntry::Function(legacy_function_abi_entry.into())
            }
            LegacyContractAbiEntry::Event(legacy_event_abi_entry) => {
                starknet_core::types::LegacyContractAbiEntry::Event(legacy_event_abi_entry.into())
            }
            LegacyContractAbiEntry::Struct(legacy_struct_abi_entry) => {
                starknet_core::types::LegacyContractAbiEntry::Struct(legacy_struct_abi_entry.into())
            }
        }
    }
}

impl From<starknet_core::types::LegacyFunctionAbiEntry> for LegacyFunctionAbiEntry {
    fn from(legacy_function_abi_entry: starknet_core::types::LegacyFunctionAbiEntry) -> Self {
        LegacyFunctionAbiEntry {
            r#type: legacy_function_abi_entry.r#type.into(),
            name: legacy_function_abi_entry.name,
            inputs: legacy_function_abi_entry.inputs.into_iter().map(|abi_entry| abi_entry.into()).collect(),
            outputs: legacy_function_abi_entry.outputs.into_iter().map(|abi_entry| abi_entry.into()).collect(),
            state_mutability: legacy_function_abi_entry
                .state_mutability
                .map(|state_mutability| state_mutability.into()),
        }
    }
}

impl From<LegacyFunctionAbiEntry> for starknet_core::types::LegacyFunctionAbiEntry {
    fn from(legacy_function_abi_entry: LegacyFunctionAbiEntry) -> Self {
        starknet_core::types::LegacyFunctionAbiEntry {
            r#type: legacy_function_abi_entry.r#type.into(),
            name: legacy_function_abi_entry.name,
            inputs: legacy_function_abi_entry.inputs.into_iter().map(|abi_entry| abi_entry.into()).collect(),
            outputs: legacy_function_abi_entry.outputs.into_iter().map(|abi_entry| abi_entry.into()).collect(),
            state_mutability: legacy_function_abi_entry
                .state_mutability
                .map(|state_mutability| state_mutability.into()),
        }
    }
}

impl From<starknet_core::types::LegacyEventAbiEntry> for LegacyEventAbiEntry {
    fn from(legacy_event_abi_entry: starknet_core::types::LegacyEventAbiEntry) -> Self {
        LegacyEventAbiEntry {
            r#type: legacy_event_abi_entry.r#type.into(),
            name: legacy_event_abi_entry.name,
            keys: legacy_event_abi_entry.keys.into_iter().map(|abi_entry| abi_entry.into()).collect(),
            data: legacy_event_abi_entry.data.into_iter().map(|abi_entry| abi_entry.into()).collect(),
        }
    }
}

impl From<LegacyEventAbiEntry> for starknet_core::types::LegacyEventAbiEntry {
    fn from(legacy_event_abi_entry: LegacyEventAbiEntry) -> Self {
        starknet_core::types::LegacyEventAbiEntry {
            r#type: legacy_event_abi_entry.r#type.into(),
            name: legacy_event_abi_entry.name,
            keys: legacy_event_abi_entry.keys.into_iter().map(|abi_entry| abi_entry.into()).collect(),
            data: legacy_event_abi_entry.data.into_iter().map(|abi_entry| abi_entry.into()).collect(),
        }
    }
}

impl From<starknet_core::types::LegacyStructAbiEntry> for LegacyStructAbiEntry {
    fn from(legacy_struct_abi_entry: starknet_core::types::LegacyStructAbiEntry) -> Self {
        LegacyStructAbiEntry {
            r#type: legacy_struct_abi_entry.r#type.into(),
            name: legacy_struct_abi_entry.name,
            size: legacy_struct_abi_entry.size,
            members: legacy_struct_abi_entry.members.into_iter().map(|member| member.into()).collect(),
        }
    }
}

impl From<LegacyStructAbiEntry> for starknet_core::types::LegacyStructAbiEntry {
    fn from(legacy_struct_abi_entry: LegacyStructAbiEntry) -> Self {
        starknet_core::types::LegacyStructAbiEntry {
            r#type: legacy_struct_abi_entry.r#type.into(),
            name: legacy_struct_abi_entry.name,
            size: legacy_struct_abi_entry.size,
            members: legacy_struct_abi_entry.members.into_iter().map(|member| member.into()).collect(),
        }
    }
}

impl From<starknet_core::types::LegacyStructMember> for LegacyStructMember {
    fn from(legacy_struct_member: starknet_core::types::LegacyStructMember) -> Self {
        LegacyStructMember {
            name: legacy_struct_member.name,
            r#type: legacy_struct_member.r#type,
            offset: legacy_struct_member.offset,
        }
    }
}

impl From<LegacyStructMember> for starknet_core::types::LegacyStructMember {
    fn from(legacy_struct_member: LegacyStructMember) -> Self {
        starknet_core::types::LegacyStructMember {
            name: legacy_struct_member.name,
            r#type: legacy_struct_member.r#type,
            offset: legacy_struct_member.offset,
        }
    }
}

impl From<starknet_core::types::LegacyTypedParameter> for LegacyTypedParameter {
    fn from(legacy_typed_parameter: starknet_core::types::LegacyTypedParameter) -> Self {
        LegacyTypedParameter { r#type: legacy_typed_parameter.r#type, name: legacy_typed_parameter.name }
    }
}

impl From<LegacyTypedParameter> for starknet_core::types::LegacyTypedParameter {
    fn from(legacy_typed_parameter: LegacyTypedParameter) -> Self {
        starknet_core::types::LegacyTypedParameter {
            r#type: legacy_typed_parameter.r#type,
            name: legacy_typed_parameter.name,
        }
    }
}

impl From<starknet_core::types::LegacyFunctionAbiType> for LegacyFunctionAbiType {
    fn from(legacy_function_abi_type: starknet_core::types::LegacyFunctionAbiType) -> Self {
        match legacy_function_abi_type {
            starknet_core::types::LegacyFunctionAbiType::Function => LegacyFunctionAbiType::Function,
            starknet_core::types::LegacyFunctionAbiType::L1Handler => LegacyFunctionAbiType::L1Handler,
            starknet_core::types::LegacyFunctionAbiType::Constructor => LegacyFunctionAbiType::Constructor,
        }
    }
}

impl From<LegacyFunctionAbiType> for starknet_core::types::LegacyFunctionAbiType {
    fn from(legacy_function_abi_type: LegacyFunctionAbiType) -> Self {
        match legacy_function_abi_type {
            LegacyFunctionAbiType::Function => starknet_core::types::LegacyFunctionAbiType::Function,
            LegacyFunctionAbiType::L1Handler => starknet_core::types::LegacyFunctionAbiType::L1Handler,
            LegacyFunctionAbiType::Constructor => starknet_core::types::LegacyFunctionAbiType::Constructor,
        }
    }
}

impl From<starknet_core::types::LegacyEventAbiType> for LegacyEventAbiType {
    fn from(legacy_event_abi_type: starknet_core::types::LegacyEventAbiType) -> Self {
        match legacy_event_abi_type {
            starknet_core::types::LegacyEventAbiType::Event => LegacyEventAbiType::Event,
        }
    }
}

impl From<LegacyEventAbiType> for starknet_core::types::LegacyEventAbiType {
    fn from(legacy_event_abi_type: LegacyEventAbiType) -> Self {
        match legacy_event_abi_type {
            LegacyEventAbiType::Event => starknet_core::types::LegacyEventAbiType::Event,
        }
    }
}

impl From<starknet_core::types::LegacyStructAbiType> for LegacyStructAbiType {
    fn from(legacy_struct_abi_type: starknet_core::types::LegacyStructAbiType) -> Self {
        match legacy_struct_abi_type {
            starknet_core::types::LegacyStructAbiType::Struct => LegacyStructAbiType::Struct,
        }
    }
}

impl From<LegacyStructAbiType> for starknet_core::types::LegacyStructAbiType {
    fn from(legacy_struct_abi_type: LegacyStructAbiType) -> Self {
        match legacy_struct_abi_type {
            LegacyStructAbiType::Struct => starknet_core::types::LegacyStructAbiType::Struct,
        }
    }
}

impl From<starknet_core::types::FunctionStateMutability> for FunctionStateMutability {
    fn from(function_state_mutability: starknet_core::types::FunctionStateMutability) -> Self {
        match function_state_mutability {
            starknet_core::types::FunctionStateMutability::View => FunctionStateMutability::View,
        }
    }
}

impl From<FunctionStateMutability> for starknet_core::types::FunctionStateMutability {
    fn from(function_state_mutability: FunctionStateMutability) -> Self {
        match function_state_mutability {
            FunctionStateMutability::View => starknet_core::types::FunctionStateMutability::View,
        }
    }
}

pub type RpcAccount<'a> = SingleOwnerAccount<&'a JsonRpcClient<HttpTransport>, LocalWallet>;
pub async fn build_single_owner_account<'a>(
    rpc: &'a JsonRpcClient<HttpTransport>,
    private_key: &str,
    account_address: &str,
    is_legacy: bool,
) -> RpcAccount<'a> {
    let signer = LocalWallet::from(SigningKey::from_secret_scalar(Felt::from_hex(private_key).unwrap()));
    let account_address = Felt::from_hex(account_address).expect("Invalid Contract Address");
    let execution_encoding = if is_legacy { ExecutionEncoding::Legacy } else { ExecutionEncoding::New };

    let chain_id = rpc.chain_id().await.unwrap();
    let mut something = SingleOwnerAccount::new(rpc, signer, account_address, chain_id, execution_encoding);
    something.set_block_id(BlockId::Tag(Pending));
    something
}

pub async fn read_erc20_balance(
    rpc: &JsonRpcClient<HttpTransport>,
    contract_address: Felt,
    account_address: Felt,
) -> Vec<Felt> {
    rpc.call(
        FunctionCall {
            contract_address,
            entry_point_selector: get_selector_from_name("balanceOf").unwrap(),
            calldata: vec![account_address],
        },
        BlockId::Tag(BlockTag::Latest),
    )
    .await
    .unwrap()
}

pub fn field_element_to_u256(input: Felt) -> U256 {
    U256::from_big_endian(&input.to_bytes_be())
}

pub fn generate_config_hash(config_hash_version: Felt, chain_id: Felt, fee_token_address: Felt) -> Felt {
    println!("checkpoint generate hash 1");
    Pedersen::hash_array(&[(config_hash_version), (chain_id), (fee_token_address)])
}

pub fn get_bridge_init_configs(config: &CliArgs) -> (Felt, Felt) {
    println!("checkpoint init config 1");
    let program_hash = Felt::from_hex(config.sn_os_program_hash.as_str()).unwrap();
    println!("checkpoint init config 2");

    println!(
        "checkpoint init config {:?}",
        Felt::from_hex(&encode(config.config_hash_version.as_str())).expect("error in config_hash_version")
    );
    println!(
        "checkpoint init config {:?}",
        Felt::from_hex(&encode(config.app_chain_id.as_str())).expect("error in app_chain_id")
    );
    println!(
        "checkpoint init config {:?}",
        Felt::from_hex(config.fee_token_address.as_str()).expect("error in fee_token_address")
    );
    let config_hash = generate_config_hash(
        Felt::from_hex(&encode(config.config_hash_version.as_str())).expect("error in config_hash_version"),
        Felt::from_hex(&encode(config.app_chain_id.as_str())).expect("error in app_chain_id"),
        Felt::from_hex(config.fee_token_address.as_str()).expect("error in fee_token_address"),
    );
    println!("checkpoint init config 3");
    (program_hash, config_hash)
}

/// Broadcasted declare contract transaction v0.
#[derive(Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BroadcastedDeclareTransactionV0 {
    /// The address of the account contract sending the declaration transaction
    pub sender_address: Felt,
    /// The maximal fee that can be charged for including the transaction
    pub max_fee: Felt,
    /// Signature
    pub signature: Vec<Felt>,
    /// The class to be declared
    pub contract_class: Arc<CompressedLegacyContractClass>,
    /// If set to `true`, uses a query-only transaction version that's invalid for execution
    pub is_query: bool,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct RpcResult<T> {
    jsonrpc: String,
    result: T,
    id: u64,
}

pub const TEMP_ACCOUNT_PRIV_KEY: &str = "0xbeef";

pub(crate) enum DeclarationInput<'a> {
    // inputs : sierra_path, casm_path
    DeclarationInputs(String, String, RpcAccount<'a>),
    // input : artifact_path
    LegacyDeclarationInputs(String, String),
}

#[allow(private_interfaces)]
pub async fn declare_contract(input: DeclarationInput<'_>) -> Felt {
    println!("checkpoint declare contract 1");
    match input {
        DeclarationInputs(sierra_path, casm_path, account) => {
            // let (class_hash, sierra) = account.declare_contract_params_sierra(&sierra_path, &casm_path);
            println!("checkpoint declare contract a.1");

            let contract_artifact: SierraClass =
                serde_json::from_reader(std::fs::File::open(sierra_path).unwrap()).unwrap();

            let contract_artifact_casm: CompiledClass =
                serde_json::from_reader(std::fs::File::open(casm_path).unwrap()).unwrap();
            let class_hash = contract_artifact_casm.class_hash().unwrap();
            let sierra_class_hash = contract_artifact.class_hash().unwrap();
            println!("checkpoint declare contract a.2");

            let flattened_class = contract_artifact.flatten().unwrap();

            account
                .declare_v3(Arc::new(flattened_class), class_hash)
                .gas(0)
                .send()
                .await
                .expect("Error in declaring the contract using Cairo 1 declaration using the provided account");
            sierra_class_hash
        }
        LegacyDeclarationInputs(artifact_path, url) => {
            println!("checkpoint declare contract b.1");
            let contract_abi_artifact_temp: LegacyContractClass = serde_json::from_reader(
                std::fs::File::open(env!("CARGO_MANIFEST_DIR").to_owned() + "/" + &artifact_path).unwrap(),
            )
            .unwrap();

            let contract_abi_artifact: CompressedLegacyContractClass = contract_abi_artifact_temp
                .clone()
                .compress()
                .expect("Error : Failed to compress the contract class")
                .into();

            println!("checkpoint declare contract b.2");
            let params: BroadcastedDeclareTransactionV0 = BroadcastedDeclareTransactionV0 {
                sender_address: Felt::from_hex("0x1").unwrap(),
                max_fee: Felt::ZERO,
                signature: Vec::new(),
                contract_class: Arc::new(contract_abi_artifact),
                is_query: false,
            };

            let json_body = &json!({
                "jsonrpc": "2.0",
                "method": "addDeclareV0Transaction",
                "params": [params],
                "id": 4
            });

            println!("checkpoint declare contract b.3");

            let req_client = reqwest::Client::new();
            let raw_txn_rpc = req_client.post(url).json(json_body).send().await;
            println!("raw_txn_rpc : {:?}", raw_txn_rpc);
            match raw_txn_rpc {
                Ok(val) => {
                    log::debug!(
                        "🚧 Txn Sent Successfully : {:?}",
                        val.json::<RpcResult<DeclareTransactionResult>>().await.unwrap()
                    );
                }
                Err(err) => {
                    log::debug!("Error : Error sending the transaction using RPC");
                    log::debug!("{:?}", err);
                }
            }

            contract_abi_artifact_temp.class_hash().unwrap()
        }
    }
}

pub(crate) async fn deploy_account_using_priv_key(
    priv_key: String,
    provider: &JsonRpcClient<HttpTransport>,
    oz_account_class_hash: Felt,
) -> Felt {
    let chain_id = provider.chain_id().await.unwrap();

    let signer = LocalWallet::from(SigningKey::from_secret_scalar(Felt::from_hex(&priv_key).unwrap()));
    log::debug!("signer : {:?}", signer);
    let mut oz_account_factory =
        OpenZeppelinAccountFactory::new(oz_account_class_hash, chain_id, signer, provider).await.unwrap();
    oz_account_factory.set_block_id(BlockId::Tag(BlockTag::Pending));
    // oz_account_factory.

    let deploy_txn = oz_account_factory.deploy_v1(Felt::ZERO).max_fee(Felt::ZERO);
    // deploy_txn.nonce(Felt::ZERO);
    let account_address = deploy_txn.address();
    log::debug!("OZ Account Deployed : {:?}", account_address);
    save_to_json("account_address", &JsonValueType::StringType(account_address.to_string())).unwrap();

    let sent_txn = deploy_txn.send().await.expect("Error in deploying the OZ account");

    log::debug!("deploy account txn_hash : {:?}", sent_txn.transaction_hash);
    wait_for_transaction(provider, sent_txn.transaction_hash, "deploy_account_using_priv_key").await.unwrap();

    account_address
}

pub(crate) async fn deploy_proxy_contract(
    account: &RpcAccount<'_>,
    account_address: Felt,
    class_hash: Felt,
    salt: Felt,
    deploy_from_zero: Felt,
) -> Felt {
    // let contract_factory = ContractFactory::new(class_hash, account);
    let txn = account
        .invoke_contract(
            account_address,
            "deploy_contract",
            vec![class_hash, salt, deploy_from_zero, Felt::ONE, Felt::ZERO],
            None,
        )
        .send()
        .await
        .expect("Error deploying the contract proxy.");

    // let txn = contract_factory
    //     .deploy_v1(vec![Felt::ONE], salt, true).max_fee(Felt::ZERO)
    // .send()
    // .await
    // .expect("Unable to deploy contract");

    log::debug!("txn in proxy contract is: {:?}", txn);

    wait_for_transaction(account.provider(), txn.transaction_hash, "deploy_proxy_contract : deploy_contract")
        .await
        .unwrap();

    log::trace!("txn hash (proxy deployment) : {:?}", txn.transaction_hash);

    let deployed_address = get_contract_address_from_deploy_tx(account.provider(), &txn).await.unwrap();
    log::trace!("[IMP] Event : {:?}", deployed_address);

    deployed_address
}

pub(crate) async fn init_governance_proxy(account: &'_ RpcAccount<'_>, contract_address: Felt, tag: &str) {
    let txn = invoke_contract(contract_address, "init_governance", vec![], account).await;
    wait_for_transaction(account.provider(), txn.transaction_hash, tag).await.unwrap();
}
