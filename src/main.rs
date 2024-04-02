pub mod utils;
pub mod felt;
pub mod messages;
pub mod snos;
pub mod bridge_deploy_utils;

use std::env;
use std::process;
use utils::arg_config::ArgConfig;

pub fn main() {

    // Reqs : 
    // ----
    // - Args :
    //      - eth_rpc
    //      - eth_priv_key
    //      - rollup_sequencer_url
    //      - rollup_priv_key

    let args: Vec<String> = env::args().collect();
    
    // args config
    let config = ArgConfig::new(&args).unwrap_or_else(|err| {
        eprintln!("Problem parsing args : {}", err);
        process::exit(1)
    });
    
}