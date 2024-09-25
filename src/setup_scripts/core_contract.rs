use std::str::FromStr;

use anyhow::Context;
use ethers::abi::Address;
use starknet_api::hash::StarkFelt;

use crate::contract_clients::config::Config;
use crate::contract_clients::core_contract::{CoreContract, CoreContractDeploy};
use crate::contract_clients::starknet_sovereign::StarknetSovereignContract;
use crate::contract_clients::starknet_validity::StarknetValidityContract;
use crate::contract_clients::utils::get_bridge_init_configs;
use crate::utils::{save_to_json, JsonValueType};
use crate::CliArgs;

pub struct CoreContractStarknetL1<'a> {
    arg_config: &'a CliArgs,
    clients: &'a Config,
}

pub struct CoreContractStarknetL1Output {
    pub core_contract_client: Box<dyn CoreContract>,
}

impl<'a> CoreContractStarknetL1<'a> {
    pub fn new(arg_config: &'a CliArgs, clients: &'a Config) -> Self {
        Self { arg_config, clients }
    }

    pub async fn setup(&self) -> anyhow::Result<CoreContractStarknetL1Output> {
        let core_contract_client: Box<dyn CoreContract> = match self.arg_config.dev {
            true => Box::new(
                StarknetSovereignContract::deploy(self.clients)
                    .await
                    .context("Deploying starknet sovereign contract")?,
            ),
            false => Box::new(
                StarknetValidityContract::deploy(self.clients).await.context("Deploying starknet validity contract")?,
            ),
        };
        log::info!("📦 Core address : {:?}", core_contract_client.address());
        save_to_json("l1_core_contract_address", &JsonValueType::EthAddress(core_contract_client.address()))
            .context("Saving the l1 core contract address to json")?;
        let (program_hash, config_hash) =
            get_bridge_init_configs(self.arg_config).context("Getting bridge initialization config")?;

        if self.arg_config.dev {
            core_contract_client
                .initialize(StarkFelt(program_hash.to_bytes_be()), config_hash)
                .await
                .context("Initializing dev core contract client")?;
        } else {
            core_contract_client
                .add_implementation_core_contract(
                    0u64.into(),
                    0u64.into(),
                    program_hash,
                    config_hash,
                    core_contract_client.implementation_address(),
                    Address::from_str(&self.arg_config.verifier_address.clone()).context("Parsing verifier_address")?,
                    false,
                )
                .await
                .context("Adding implementation core contract")?;
            core_contract_client
                .upgrade_to_core_contract(
                    0u64.into(),
                    0u64.into(),
                    program_hash,
                    config_hash,
                    core_contract_client.implementation_address(),
                    Address::from_str(&self.arg_config.verifier_address.clone()).context("Parsing verifier_address")?,
                    false,
                )
                .await
                .context("Upgrading core contract")?;
            core_contract_client
                .register_operator_core_contract(
                    Address::from_str(&self.arg_config.operator_address.clone()).context("Parsing operator_address")?,
                )
                .await
                .context("Registering operator address")?;

            let l1_multisig_addr = Address::from_str(&self.arg_config.l1_multisig_address.clone())
                .context("Parsing L1 multisig address")?;

            core_contract_client
                .nominate_governor_core_contract(l1_multisig_addr)
                .await
                .context("Nominator governor")?;
            core_contract_client
                .nominate_governor_core_contract_proxy(l1_multisig_addr)
                .await
                .context("Nominator governor contract proxy")?;
        }

        Ok(CoreContractStarknetL1Output { core_contract_client })
    }
}
