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
}

impl ArgConfig {
    pub fn new(args: &[String]) -> Result<ArgConfig, &str> {
        if args.len() < 4 {
            return Err("Not Enough params. Required params : eth_rpc, eth_priv_key, rollup_seq_url, rollup_priv_key, l1_deployer_address")
        }

        let eth_rpc = args[1].clone();
        let eth_priv_key = args[2].clone();
        let rollup_seq_url = args[3].clone();
        let rollup_priv_key = args[4].clone();
        let eth_chain_id = args[5].clone().parse::<u64>().unwrap();
        let l1_deployer_address = args[6].clone();
        let l2_deployer_address = args[7].clone();

        Ok(ArgConfig {eth_rpc, eth_priv_key, rollup_seq_url, rollup_priv_key, eth_chain_id, l1_deployer_address, l2_deployer_address})
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