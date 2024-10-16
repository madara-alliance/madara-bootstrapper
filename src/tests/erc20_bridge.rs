use std::str::FromStr;
use std::time::Duration;

use ethers::prelude::U256;
use ethers::types::Address;
use starknet::core::types::Felt;
use tokio::time::sleep;

use crate::contract_clients::config::Config;
use crate::contract_clients::token_bridge::StarknetTokenBridge;
use crate::contract_clients::utils::{build_single_owner_account, read_erc20_balance};
use crate::tests::constants::L2_DEPLOYER_ADDRESS;
use crate::utils::invoke_contract;
use crate::CliArgs;
// TODO : Uncomment L2 --> L1 part after this part is added in the madara-orchestrator.

pub async fn erc20_bridge_test_helper(
    clients: &Config,
    arg_config: &CliArgs,
    l2_erc20_token_address: Felt,
    token_bridge: StarknetTokenBridge,
    l2_bridge_address: Felt,
) -> Result<(), anyhow::Error> {
    token_bridge.approve(token_bridge.bridge_address(), 100000000.into()).await;
    sleep(Duration::from_secs(arg_config.l1_wait_time.parse().unwrap())).await;
    log::debug!("Approval done [✅]");
    log::debug!("Waiting for message to be consumed on l2 [⏳]");
    sleep(Duration::from_secs(arg_config.cross_chain_wait_time)).await;

    let balance_before =
        read_erc20_balance(clients.provider_l2(), l2_erc20_token_address, Felt::from_str(L2_DEPLOYER_ADDRESS).unwrap())
            .await;

    token_bridge
        .deposit(
            token_bridge.address(),
            10.into(),
            U256::from_str(L2_DEPLOYER_ADDRESS).unwrap(),
            U256::from_dec_str("100000000000000").unwrap(),
        )
        .await;
    sleep(Duration::from_secs(arg_config.l1_wait_time.parse().unwrap())).await;
    log::debug!("Deposit done [💰]");
    log::debug!("Waiting for message to be consumed on l2 [⏳]");
    sleep(Duration::from_secs(arg_config.cross_chain_wait_time)).await;

    let balance_after =
        read_erc20_balance(clients.provider_l2(), l2_erc20_token_address, Felt::from_str(L2_DEPLOYER_ADDRESS).unwrap())
            .await;

    assert_eq!(balance_before[0] + Felt::from(10), balance_after[0]);

    // let l1_recipient = Felt::from_hex(&arg_config.l1_deployer_address).unwrap();
    // let account =
    //     build_single_owner_account(clients.provider_l2(), &arg_config.rollup_priv_key, L2_DEPLOYER_ADDRESS, false)
    //         .await;

    // log::debug!("Initiated token withdraw on L2 [⏳]");
    // invoke_contract(
    //     l2_bridge_address,
    //     "initiate_token_withdraw",
    //     vec![
    //         Felt::from_bytes_be_slice(token_bridge.address().as_bytes()),
    //         l1_recipient,
    //         Felt::from_dec_str("5").unwrap(),
    //         Felt::ZERO,
    //     ],
    //     &account,
    // )
    // .await;

    // sleep(Duration::from_secs(arg_config.l1_wait_time.parse().unwrap())).await;
    // log::debug!("Waiting for message to be consumed on l2 [⏳]");
    // sleep(Duration::from_secs(arg_config.cross_chain_wait_time)).await;
    // sleep(Duration::from_secs(arg_config.l1_wait_time.parse().unwrap())).await;

    // let l1_recipient: Address = Address::from_str(&arg_config.l1_deployer_address).unwrap();
    // let balance_before = token_bridge.token_balance(l1_recipient).await;
    // token_bridge.withdraw(token_bridge.address(), 5.into(), l1_recipient).await;
    // let balance_after = token_bridge.token_balance(l1_recipient).await;

    // assert_eq!(balance_before + U256::from_dec_str("5").unwrap(), balance_after);

    // log::debug!("Token withdraw successful [✅]");

    anyhow::Ok(())
}
