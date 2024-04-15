use ethereum_instance::EthereumInstance;
use starknet_providers::jsonrpc::HttpTransport;
use starknet_providers::JsonRpcClient;
use url::Url;

use crate::CliArgs;

pub struct Config {
    eth_client: EthereumInstance,
    provider_l2: JsonRpcClient<HttpTransport>,
}

impl Config {
    pub fn provider_l2(&self) -> &JsonRpcClient<HttpTransport> {
        &self.provider_l2
    }

    pub fn eth_client(&self) -> &EthereumInstance {
        &self.eth_client
    }

    /// To deploy the instance of ethereum and starknet and returning the struct.
    pub async fn init(config: &CliArgs) -> Self {
        let client_instance =
            EthereumInstance::spawn(config.eth_rpc.clone(), config.eth_priv_key.clone(), config.eth_chain_id);
        let provider_l2 = JsonRpcClient::new(HttpTransport::new(
            Url::parse(&config.rollup_seq_url).expect("Failed to declare provider for app chain"),
        ));

        Self { eth_client: client_instance, provider_l2 }
    }
}
