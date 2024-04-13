use std::sync::Arc;
use crate::felt::lib::Felt252Wrapper;
use starknet_api::hash::{StarkFelt};
use starknet_core_contract_client::clients::StarknetSovereignContractClient;
use starknet_core_contract_client::deploy_starknet_sovereign_behind_unsafe_proxy;
use starknet_core_contract_client::interfaces::{OperatorTrait};
use starknet_proxy_client::proxy_support::{
    CoreContractInitData, CoreContractState, ProxyInitializeData, ProxySupportTrait,
};
use zaun_utils::{LocalWalletSignerMiddleware, StarknetContractClient};
use ethereum_instance::EthereumInstance;
use ethers::{types::{Address, I256}};
use starknet_accounts::SingleOwnerAccount;
use starknet_core::chain_id;
use starknet_core::types::{BlockId, BlockTag, FunctionCall};
use starknet_core::utils::get_selector_from_name;
use crate::ArgConfig;
use starknet_ff::FieldElement;
use starknet_providers::jsonrpc::HttpTransport;
use starknet_providers::JsonRpcClient;
use starknet_signers::{LocalWallet, SigningKey};
use url::Url;
use crate::utils::convert_felt_to_u256;
use starknet_providers::Provider;


pub type RpcAccount<'a> = SingleOwnerAccount<&'a JsonRpcClient<HttpTransport>, LocalWallet>;
pub fn build_single_owner_account<'a>(
    rpc: &'a JsonRpcClient<HttpTransport>,
    private_key: &str,
    account_address: &str,
    is_legacy: bool,
) -> RpcAccount<'a> {
    let signer = LocalWallet::from(SigningKey::from_secret_scalar(FieldElement::from_hex_be(private_key).unwrap()));
    let account_address = FieldElement::from_hex_be(account_address).expect("Invalid Contract Address");
    let execution_encoding = if is_legacy {
        starknet_accounts::ExecutionEncoding::Legacy
    } else {
        starknet_accounts::ExecutionEncoding::New
    };
    SingleOwnerAccount::new(rpc, signer, account_address, chain_id::TESTNET, execution_encoding)
}


pub async fn read_erc20_balance(
    rpc: &JsonRpcClient<HttpTransport>,
    contract_address: FieldElement,
    account_address: FieldElement,
) -> Vec<FieldElement> {
    let balance = rpc.call(
        FunctionCall {
            contract_address,
            entry_point_selector: get_selector_from_name("balanceOf").unwrap(),
            calldata: vec![account_address],
        },
        BlockId::Tag(BlockTag::Latest)
    ).await.unwrap();

    balance
}

#[allow(dead_code)]
pub struct Config {
    eth_client: EthereumInstance,
    client: StarknetSovereignContractClient,
    provider_l2: JsonRpcClient<HttpTransport>
}

impl Config {

    pub fn address(&self) -> Address {
        self.client.address()
    }

    pub fn client(&self) -> Arc<LocalWalletSignerMiddleware> {
        self.client.client()
    }

    pub fn provider_l2(&self) -> &JsonRpcClient<HttpTransport> { &self.provider_l2 }

    /// To deploy the instance of ethereum and starknet and returning the struct.
    pub async fn deploy(config: &ArgConfig) -> Self {
        let client_instance = EthereumInstance::spawn(config.eth_rpc.clone(), config.eth_priv_key.clone(), config.eth_chain_id);
        let provider_l2 = JsonRpcClient::new(HttpTransport::new(Url::parse(&config.rollup_seq_url).expect("Failed to declare provider for app chain")));
        let client = deploy_starknet_sovereign_behind_unsafe_proxy(client_instance.client()).await.expect("Failed to deploy the starknet contact");

        Self {
            eth_client: client_instance,
            client,
            provider_l2
        }
    }

    /// Initialize Starknet core contract with the specified data.
    /// Also register Anvil default account as an operator.
    pub async fn initialize_with(&self, init_data: CoreContractInitData) {
        let data = ProxyInitializeData::<0> { sub_contract_addresses: [], eic_address: Default::default(), init_data };

        self.client.initialize_with(data).await.expect("Failed to initialize");

        self.client.register_operator(self.client.client().address()).await.expect("Failed to register operator");
    }

    /// Initialize Starknet core contract with the specified program and config hashes. The rest of parameters will be left default.
    /// Also register Anvil default account as an operator.
    pub async fn initialize(&self, program_hash: StarkFelt, config_hash: StarkFelt) {
        self.initialize_with(CoreContractInitData {
            program_hash: convert_felt_to_u256(program_hash),
            config_hash: convert_felt_to_u256(config_hash),
            ..Default::default()
        })
        .await;
    }

    /// Initialize Starknet core contract with the specified block number and state root hash.
    /// The program and config hashes will be set according to the Madara Goerli configuration.
    /// Also register Anvil default account as an operator.
    pub async fn initialize_for_goerli(&self, block_number: StarkFelt, state_root: StarkFelt) {
        // See SN_OS_PROGRAM_HASH constant
        let program_hash = StarkFelt::from(Felt252Wrapper::from(
            FieldElement::from_hex_be("0x41fc2a467ef8649580631912517edcab7674173f1dbfa2e9b64fbcd82bc4d79").unwrap(),
        ));

        // Hash version:        SN_OS_CONFIG_HASH_VERSION (settlement)
        // Chain ID:            SN_GOERLI_CHAIN_ID (pallet config)
        // Fee token address:   0x49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7 (genesis
        // config)
        let config_hash = StarkFelt::from(Felt252Wrapper::from(
            FieldElement::from_hex_be("0x05ac6b99d1ab6d37202e29e2c887ace63cc594b40f900cf2c47398272bef412c").unwrap(),
        ));

        let init_data = CoreContractInitData {
            program_hash: convert_felt_to_u256(program_hash), // zero program hash would be deemed invalid
            config_hash: convert_felt_to_u256(config_hash),
            // TODO :
            // Figure out the exact params for production env
            initial_state: CoreContractState {
                block_number: I256::from_raw(convert_felt_to_u256(block_number)),
                state_root: convert_felt_to_u256(state_root),
                ..Default::default()
            },
            ..Default::default()
        };

        self.initialize_with(init_data).await;
    }
}