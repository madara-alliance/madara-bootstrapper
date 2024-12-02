use std::time::Duration;

use starknet::accounts::{Account, ConnectedAccount};
use starknet_providers::jsonrpc::HttpTransport;
use starknet_providers::JsonRpcClient;
use starknet_types_core::felt::Felt;
use tokio::time::sleep;

use crate::contract_clients::utils::{declare_contract, DeclarationInput, RpcAccount};
use crate::helpers::account_actions::{get_contract_address_from_deploy_tx, AccountActions};
use crate::utils::constants::{
    EIC_ETH_BRIDGE_CASM_PATH, EIC_ETH_BRIDGE_SIERRA_PATH, NEW_ETH_BRIDGE_CASM_PATH, NEW_ETH_BRIDGE_SIERRA_PATH,
};
use crate::utils::wait_for_transaction;

pub async fn upgrade_eth_bridge_to_cairo_1(
    account: &RpcAccount<'_>,
    rpc_provider_l2: &JsonRpcClient<HttpTransport>,
    l2_eth_bridge_address: Felt,
) {
    let eth_bridge_eic_class_hash = declare_contract(DeclarationInput::DeclarationInputs(
        String::from(EIC_ETH_BRIDGE_SIERRA_PATH),
        String::from(EIC_ETH_BRIDGE_CASM_PATH),
        account.clone(),
    ))
    .await;
    sleep(Duration::from_secs(5)).await;
    log::debug!("ETH Bridge EIC declared ✅, Class hash : {:?}", eth_bridge_eic_class_hash);

    let new_eth_bridge_class_hash = declare_contract(DeclarationInput::DeclarationInputs(
        String::from(NEW_ETH_BRIDGE_SIERRA_PATH),
        String::from(NEW_ETH_BRIDGE_CASM_PATH),
        account.clone(),
    ))
    .await;
    sleep(Duration::from_secs(5)).await;
    log::debug!("New ETH Bridge declared ✅, Class hash : {:?}", new_eth_bridge_class_hash);

    let bridge_eic_deploy_tx = account
        .invoke_contract(
            account.address(),
            "deploy_contract",
            vec![eth_bridge_eic_class_hash, Felt::ZERO, Felt::ZERO, Felt::ZERO],
            None,
        )
        .send()
        .await
        .expect("Error deploying the contract : eth_eic_deploy_tx");
    wait_for_transaction(rpc_provider_l2, bridge_eic_deploy_tx.transaction_hash, " : deploy").await.unwrap();
    let eth_bridge_eic_contract_address =
        get_contract_address_from_deploy_tx(account.provider(), &bridge_eic_deploy_tx).await.unwrap();
    log::debug!("✅ eth bridge eic contract address : {:?}", eth_bridge_eic_contract_address);
    sleep(Duration::from_secs(5)).await;

    let new_bridge_eth_deploy_tx = account
        .invoke_contract(
            account.address(),
            "deploy_contract",
            vec![new_eth_bridge_class_hash, Felt::ONE, Felt::ZERO, Felt::from(1u64), Felt::from(0)],
            None,
        )
        .send()
        .await
        .expect("Error deploying the contract : new_token_eth_deploy_tx");
    wait_for_transaction(rpc_provider_l2, new_bridge_eth_deploy_tx.transaction_hash, " : deploy").await.unwrap();
    let new_eth_bridge_contract_address =
        get_contract_address_from_deploy_tx(account.provider(), &new_bridge_eth_deploy_tx).await.unwrap();
    log::debug!("✅ new eth bridge contract address : {:?}", new_eth_bridge_contract_address);
    sleep(Duration::from_secs(5)).await;

    let txn_1 = account
        .invoke_contract(
            l2_eth_bridge_address,
            "add_implementation",
            vec![
                new_eth_bridge_contract_address,
                eth_bridge_eic_contract_address,
                Felt::TWO,
                Felt::from_hex("455448").unwrap(),
                Felt::from_hex("49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7").unwrap(),
                Felt::ZERO,
            ],
            None,
        )
        .send()
        .await
        .expect("Error calling eth token proxy");
    wait_for_transaction(rpc_provider_l2, txn_1.transaction_hash, "Interact ETH bridge").await.unwrap();
    log::debug!(
        "upgrade_eth_bridge_to_cairo_1 : add_implementation : eth bridge ✅, Txn hash : {:?}",
        txn_1.transaction_hash
    );

    let txn_2 = account
        .invoke_contract(
            l2_eth_bridge_address,
            "upgrade_to",
            vec![
                new_eth_bridge_contract_address,
                eth_bridge_eic_contract_address,
                Felt::TWO,
                Felt::from_hex("455448").unwrap(),
                Felt::from_hex("49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7").unwrap(),
                Felt::ZERO,
            ],
            None,
        )
        .send()
        .await
        .expect("Error calling eth token proxy");
    wait_for_transaction(rpc_provider_l2, txn_2.transaction_hash, "Interact ETH bridge").await.unwrap();
    log::debug!("upgrade_eth_bridge_to_cairo_1 : upgrade_to : eth bridge ✅, Txn hash : {:?}", txn_2.transaction_hash);

    let txn_5 = account
        .invoke_contract(l2_eth_bridge_address, "register_governance_admin", vec![account.address()], None)
        .send()
        .await
        .expect("Error calling eth token proxy");
    wait_for_transaction(rpc_provider_l2, txn_5.transaction_hash, "Interact ETH bridge").await.unwrap();
    log::debug!(
        "upgrade_eth_bridge_to_cairo_1 : register_governance_admin : eth bridge ✅, Txn hash : {:?}",
        txn_5.transaction_hash
    );

    let txn_6 = account
        .invoke_contract(l2_eth_bridge_address, "register_upgrade_governor", vec![account.address()], None)
        .send()
        .await
        .expect("Error calling eth token proxy");
    wait_for_transaction(rpc_provider_l2, txn_6.transaction_hash, "Interact ETH bridge").await.unwrap();
    log::debug!(
        "upgrade_eth_bridge_to_cairo_1 : register_upgrade_governor : eth bridge ✅, Txn hash : {:?}",
        txn_6.transaction_hash
    );

    let txn_3 = account
        .invoke_contract(
            l2_eth_bridge_address,
            "add_new_implementation",
            vec![new_eth_bridge_class_hash, Felt::ONE, Felt::ZERO],
            None,
        )
        .send()
        .await
        .expect("Error calling eth token proxy");
    wait_for_transaction(rpc_provider_l2, txn_3.transaction_hash, "Interact ETH token").await.unwrap();
    log::debug!(
        "upgrade_eth_bridge_to_cairo_1 : add_new_implementation : eth bridge ✅, Txn hash : {:?}",
        txn_3.transaction_hash
    );

    let txn_4 = account
        .invoke_contract(
            l2_eth_bridge_address,
            "replace_to",
            vec![new_eth_bridge_class_hash, Felt::ONE, Felt::ZERO],
            None,
        )
        .send()
        .await
        .expect("Error calling eth token proxy");
    wait_for_transaction(rpc_provider_l2, txn_4.transaction_hash, "Interact ETH token").await.unwrap();
    log::debug!("upgrade_eth_bridge_to_cairo_1 : replace_to : eth bridge ✅, Txn hash : {:?}", txn_4.transaction_hash);

    log::info!("Eth bridge L2 upgraded successfully ✅");
}
