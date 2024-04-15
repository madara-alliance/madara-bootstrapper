use crate::utils::constants::{
    APP_CHAIN_ID, ETH_CHAIN_ID, ETH_PRIV_KEY, ETH_RPC, FEE_TOKEN_ADDRESS, L1_DEPLOYER_ADDRESS,
    L1_WAIT_TIME, L2_DEPLOYER_ADDRESS, ROLLUP_PRIV_KEY, ROLLUP_SEQ_URL, SN_OS_CONFIG_HASH_VERSION,
    SN_OS_PROGRAM_HASH,
};
use crate::CliArgs;
use starknet_ff::FieldElement;

#[derive(Clone)]
pub struct ArgConfig {
    pub eth_rpc: String,
    pub eth_priv_key: String,
    pub rollup_seq_url: String,
    pub rollup_priv_key: String,
    pub eth_chain_id: u64,
    pub l1_deployer_address: String,
    pub l2_deployer_address: String,
    pub l1_wait_time: String,
    pub sn_os_program_hash: String,
    pub config_hash_version: String,
    pub app_chain_id: String,
    pub fee_token_address: String,
}

impl ArgConfig {
    pub fn new(args: &CliArgs) -> Result<ArgConfig, &str> {
        let eth_rpc = args.eth_rpc.clone();
        let eth_priv_key = args.eth_priv_key.clone();
        let rollup_seq_url = args.rollup_seq_url.clone();
        let rollup_priv_key = args.rollup_priv_key.clone();
        let eth_chain_id = args.eth_chain_id.clone();
        let l1_deployer_address = args.l1_deployer_address.clone();
        let l2_deployer_address = args.l2_deployer_address.clone();
        let l1_wait_time = args.l1_wait_time.clone();
        let sn_os_program_hash = args.sn_os_program_hash.clone();
        let config_hash_version = args.config_hash_version.clone();
        let app_chain_id = args.app_chain_id.clone();
        let fee_token_address = args.fee_token_address.clone();

        Ok(ArgConfig {
            eth_rpc,
            eth_priv_key,
            rollup_seq_url,
            rollup_priv_key,
            eth_chain_id,
            l1_deployer_address,
            l2_deployer_address,
            l1_wait_time,
            sn_os_program_hash,
            config_hash_version,
            app_chain_id,
            fee_token_address,
        })
    }

    pub fn test<'a>() -> Result<ArgConfig, &'a str> {
        Ok(ArgConfig {
            eth_rpc: String::from(ETH_RPC),
            eth_priv_key: String::from(ETH_PRIV_KEY),
            rollup_seq_url: String::from(ROLLUP_SEQ_URL),
            rollup_priv_key: String::from(ROLLUP_PRIV_KEY),
            eth_chain_id: String::from(ETH_CHAIN_ID).parse().unwrap(),
            l1_deployer_address: String::from(L1_DEPLOYER_ADDRESS),
            l2_deployer_address: String::from(L2_DEPLOYER_ADDRESS),
            l1_wait_time: String::from(L1_WAIT_TIME),
            sn_os_program_hash: String::from(SN_OS_PROGRAM_HASH),
            config_hash_version: String::from(SN_OS_CONFIG_HASH_VERSION),
            app_chain_id: String::from(APP_CHAIN_ID),
            fee_token_address: String::from(FEE_TOKEN_ADDRESS),
        })
    }
}

#[derive(Clone)]
pub struct TestAddressesStruct {
    pub l1_eth_bridge_address: String,
    pub l2_eth_bridge_address: FieldElement,
    pub l2_eth_token_address: FieldElement,
    pub l1_token_bridge_address: String,
    pub l1_token_address: String,
    pub l2_token_bridge_address: FieldElement,
    pub l2_token_address: FieldElement,
}

impl TestAddressesStruct {
    pub fn new() -> Self {
        Self {
            l1_eth_bridge_address: String::from(""),
            l2_eth_bridge_address: FieldElement::ZERO,
            l2_eth_token_address: FieldElement::ZERO,
            l1_token_bridge_address: String::from(""),
            l1_token_address: String::from(""),
            l2_token_bridge_address: FieldElement::ZERO,
            l2_token_address: FieldElement::ZERO,
        }
    }

    pub fn fetch(self) -> Self {
        self
    }

    pub fn set_l1_eth_bridge_address(mut self, address: String) {
        self.l1_eth_bridge_address = address;
    }

    pub fn set_l2_eth_bridge_address(mut self, address: FieldElement) {
        self.l2_eth_bridge_address = address;
    }

    pub fn set_l2_eth_token_address(mut self, address: FieldElement) {
        self.l2_eth_token_address = address;
    }

    pub fn set_l1_token_bridge_address(mut self, address: String) {
        self.l1_token_bridge_address = address;
    }

    pub fn set_l1_token_address(mut self, address: String) {
        self.l1_token_address = address;
    }

    pub fn set_l2_token_bridge_address(mut self, address: FieldElement) {
        self.l2_token_bridge_address = address;
    }

    pub fn set_l2_token_address(mut self, address: FieldElement) {
        self.l2_token_address = address;
    }
}
