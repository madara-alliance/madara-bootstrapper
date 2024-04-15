use crate::tests::constants::{
    APP_CHAIN_ID, ETH_CHAIN_ID, ETH_PRIV_KEY, ETH_RPC, FEE_TOKEN_ADDRESS, L1_DEPLOYER_ADDRESS, L1_WAIT_TIME,
    L2_DEPLOYER_ADDRESS, ROLLUP_PRIV_KEY, ROLLUP_SEQ_URL, SN_OS_CONFIG_HASH_VERSION, SN_OS_PROGRAM_HASH,
};
use crate::CliArgs;

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
    pub fn test<'a>() -> Result<CliArgs, &'a str> {
        Ok(CliArgs {
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
