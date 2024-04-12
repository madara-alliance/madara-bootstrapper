use starknet_ff::FieldElement;
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

        Ok(ArgConfig {eth_rpc, eth_priv_key, rollup_seq_url, rollup_priv_key, eth_chain_id, l1_deployer_address, l2_deployer_address, l1_wait_time})
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
    pub l2_token_address: FieldElement
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