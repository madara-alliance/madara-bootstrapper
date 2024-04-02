pub struct ArgConfig {
    pub eth_rpc: String,
    pub eth_priv_key: String,
    pub rollup_seq_url: String,
    pub rollup_priv_key: String,
    pub eth_chain_id: u64,
}

impl ArgConfig {
    pub fn new(args: &[String]) -> Result<ArgConfig, &str> {
        if args.len() < 4 {
            return Err("Not Enough params. Required params : eth_rpc, eth_priv_key, rollup_seq_url, rollup_priv_key")
        }

        let eth_rpc = args[1].clone();
        let eth_priv_key = args[2].clone();
        let rollup_seq_url = args[3].clone();
        let rollup_priv_key = args[4].clone();
        let eth_chain_id = args[5].clone().parse::<u64>().unwrap();

        Ok(ArgConfig {eth_rpc, eth_priv_key, rollup_seq_url, rollup_priv_key, eth_chain_id})
    }
}