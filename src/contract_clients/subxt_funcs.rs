#![allow(missing_docs)]

use starknet_core::types::contract::legacy::LegacyContractClass;
use starknet_ff::FieldElement;
use subxt::{OnlineClient, PolkadotConfig, SubstrateConfig};

use crate::contract_clients::subxt_funcs::appchain::runtime_types::blockifier::execution::contract_class::{
    ClassInfo, ContractClass, ContractClassV0, ContractClassV0Inner,
};
use crate::contract_clients::subxt_funcs::appchain::runtime_types::blockifier::transaction::transactions::DeclareTransaction;
use crate::contract_clients::subxt_funcs::appchain::runtime_types::cairo_vm::types::program::Program;
use crate::contract_clients::subxt_funcs::appchain::runtime_types::starknet_api::core::{
    ClassHash, ContractAddress, Nonce,
};
use crate::contract_clients::subxt_funcs::appchain::runtime_types::starknet_api::deprecated_contract_class::{
    EntryPoint, EntryPointType,
};
use crate::contract_clients::subxt_funcs::appchain::runtime_types::starknet_api::hash::StarkFelt;
use crate::contract_clients::subxt_funcs::appchain::runtime_types::starknet_api::transaction::{
    DeclareTransactionV0V1, Fee, TransactionHash, TransactionSignature,
};




#[subxt::subxt(runtime_metadata_path = "./src/artifacts/madara.artifact.scale")]
pub mod appchain {}

// struct StarknetConfig {
// }
//
// impl StarknetConfig {
//     type ExtrinsicParams = appchain::ExtrinsicParams;
// }

pub async fn declare_contract_subxt(declare_txn: DeclareTransaction) -> Result<(), Box<dyn std::error::Error>> {
    let api = OnlineClient::<SubstrateConfig>::new().await.unwrap();
    let declare_call = appchain::tx().starknet().declare(declare_txn);
    api.tx().create_unsigned(&declare_call).unwrap().submit_and_watch().await.unwrap();
    Ok(())
}

pub async fn toggle_fee(val: bool) -> Result<(), Box<dyn std::error::Error>> {
    let api = OnlineClient::<SubstrateConfig>::new().await?;
    let fee_call = appchain::tx().starknet().set_disable_fee(val);
    api.tx().create_unsigned(&fee_call).unwrap().submit_and_watch().await.unwrap();
    Ok(())
}

pub async fn declare_transaction_build_subxt(
    class_hash: FieldElement,
    txn_hash: FieldElement,
    artifact_reqs: LegacyContractClass,
    artifact_program: Program,
    entrypoints_artifact: Vec<(EntryPointType, Vec<EntryPoint>)>,
) {
    let blank_sig: Vec<StarkFelt> = Vec::new();
    let empty_array: [u8; 32] = [0; 32];

    let txn: DeclareTransaction = DeclareTransaction {
        tx: appchain::runtime_types::starknet_api::transaction::DeclareTransaction::V0(DeclareTransactionV0V1 {
            max_fee: Fee(0u128),
            signature: TransactionSignature(blank_sig),
            nonce: Nonce(StarkFelt(empty_array)),
            class_hash: ClassHash(StarkFelt(class_hash.to_bytes_be())),
            sender_address: ContractAddress(appchain::runtime_types::starknet_api::core::PatriciaKey(StarkFelt(
                FieldElement::ONE.to_bytes_be(),
            ))),
        }),
        tx_hash: TransactionHash(StarkFelt(txn_hash.to_bytes_be())),
        only_query: false,
        class_info: ClassInfo {
            contract_class: ContractClass::V0(ContractClassV0 {
                0: ContractClassV0Inner { program: artifact_program, entry_points_by_type: entrypoints_artifact },
            }),
            sierra_program_length: 0,
            abi_length: artifact_reqs.abi.len() as u64,
        },
    };

    declare_contract_subxt(txn).await.unwrap();
}
