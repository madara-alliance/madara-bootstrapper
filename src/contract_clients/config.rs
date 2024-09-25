use anyhow::Context;
use ethereum_instance::EthereumClient;
use starknet_providers::jsonrpc::HttpTransport;
use starknet_providers::JsonRpcClient;
use url::Url;

use crate::CliArgs;

pub struct Config {
    eth_client: EthereumClient,
    provider_l2: JsonRpcClient<HttpTransport>,
}

impl Config {
    pub fn provider_l2(&self) -> &JsonRpcClient<HttpTransport> {
        &self.provider_l2
    }

    pub fn eth_client(&self) -> &EthereumClient {
        &self.eth_client
    }

    /// To deploy the instance of ethereum and starknet and returning the struct.
    pub async fn init(config: &CliArgs) -> anyhow::Result<Self> {
        let client_instance = EthereumClient::attach(
            Some(config.eth_rpc.clone()),
            Some(config.eth_priv_key.clone()),
            Some(config.eth_chain_id),
        )
        .context("Creating the Ethereum RPC client")?;

        let provider_l2 = JsonRpcClient::new(HttpTransport::new(
            Url::parse(&config.rollup_seq_url).expect("Failed to declare provider for app chain"),
        ));

        Ok(Self { eth_client: client_instance, provider_l2 })
    }
}
