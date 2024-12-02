use std::str::FromStr;
use std::sync::Arc;

use ethereum_instance::Error;
use ethers::prelude::{abigen, Bytes, SignerMiddleware};
use ethers::providers::{Http, Provider};
use ethers::signers::{LocalWallet, Signer};
use ethers::types::{Address, U256};

use crate::ConfigFile;

abigen!(
    EthereumL1BridgeProxy,
    r"[
        function addImplementation(address newImplementation, bytes data, bool finalize)
        function upgradeTo(address newImplementation, bytes data, bool finalize)
    ]",
);

abigen!(EthereumNewBridge, "src/upgrade-artifacts/eth_bridge_upgraded.json");
abigen!(EthereumNewBridgeEIC, "src/upgrade-artifacts/eic_eth_bridge.json");

pub async fn upgrade_l1_bridge(ethereum_bridge_address: Address, config_file: &ConfigFile) -> color_eyre::Result<()> {
    let config_file = Arc::from(config_file);

    let provider = Provider::<Http>::try_from(config_file.eth_rpc.clone()).map_err(|_| Error::UrlParser)?;
    let wallet: LocalWallet = config_file.eth_priv_key.parse().expect("Failed to parse private key");
    let signer_client =
        Arc::new(SignerMiddleware::new(provider.clone(), wallet.with_chain_id(config_file.eth_chain_id)));

    let new_eth_bridge_client = EthereumNewBridge::deploy(signer_client.clone(), ())?.send().await?;
    log::debug!("New ETH bridge deployed : {:?}", new_eth_bridge_client.address());
    let eic_eth_bridge_client = EthereumNewBridgeEIC::deploy(signer_client.clone(), ())?.send().await?;
    log::debug!("New ETH bridge EIC deployed : {:?}", eic_eth_bridge_client.address());

    let eth_bridge_proxy_client = EthereumL1BridgeProxy::new(ethereum_bridge_address, signer_client.clone());

    // Building calldata :
    let client_clone = eic_eth_bridge_client.clone();
    let address = client_clone.address();
    let eic_eth_bridge_bytes = address.as_ref();
    // let eic_eth_bridge_bytes = eic_eth_bridge_client.address().as_ref();
    let mut padded_eic_eth_bridge_address = Vec::with_capacity(32);
    padded_eic_eth_bridge_address.extend(vec![0u8; 32 - eic_eth_bridge_bytes.len()]);
    padded_eic_eth_bridge_address.extend_from_slice(eic_eth_bridge_bytes);
    let empty_bytes = Bytes::from_str("0000000000000000000000000000000000000000000000000000000000000000")?;
    let call_data = [padded_eic_eth_bridge_address, empty_bytes.to_vec(), empty_bytes.to_vec()].concat();
    let call_data = Bytes::from(call_data);

    eth_bridge_proxy_client
        .add_implementation(new_eth_bridge_client.address(), call_data.clone(), false)
        .send()
        .await?;
    log::debug!("New ETH bridge add_implementation ✅");
    eth_bridge_proxy_client.upgrade_to(new_eth_bridge_client.address(), call_data, false).send().await?;
    log::debug!("New ETH bridge upgrade_to ✅");
    new_eth_bridge_client
        .set_max_total_balance(
            Address::from_str("0x0000000000000000000000000000000000455448").unwrap(),
            U256::from_dec_str("10000000000000000000000000").unwrap(),
        )
        .send()
        .await?;
    log::debug!("New ETH bridge set_max_total_balance ✅");

    log::info!("Eth bridge L1 upgraded successfully ✅");
    Ok(())
}
