use std::str::FromStr;

use ethers::abi::Address;

use crate::contract_clients::config::Clients;
use crate::contract_clients::core_contract::{CoreContract, CoreContractDeploy};
use crate::contract_clients::starknet_core_contract::StarknetCoreContract;
use crate::contract_clients::starknet_dev_core_contract::StarknetDevCoreContract;
use crate::contract_clients::utils::get_bridge_init_configs;
use crate::utils::{save_to_json, JsonValueType};
use crate::{ConfigFile, CoreContractMode};

pub struct CoreContractStarknetL1<'a> {
    arg_config: &'a ConfigFile,
    clients: &'a Clients,
}

pub struct CoreContractStarknetL1Output {
    pub core_contract_client: Box<dyn CoreContract>,
}

impl<'a> CoreContractStarknetL1<'a> {
    pub fn new(arg_config: &'a ConfigFile, clients: &'a Clients) -> Self {
        Self { arg_config, clients }
    }

    pub async fn setup(&self) -> CoreContractStarknetL1Output {
        let core_contract_client: Box<dyn CoreContract> = match self.arg_config.core_contract_mode {
            CoreContractMode::Production => Box::new(StarknetCoreContract::deploy(self.clients).await),
            CoreContractMode::Dev => Box::new(StarknetDevCoreContract::deploy(self.clients).await),
        };
        log::info!("📦 Core address : {:?}", core_contract_client.address());

        save_to_json("l1_core_contract_address", &JsonValueType::EthAddress(core_contract_client.address())).unwrap();
        let (program_hash, config_hash) = get_bridge_init_configs(self.arg_config);

        core_contract_client
            .add_implementation_core_contract(
                0u64.into(),
                0u64.into(),
                0u64.into(),
                program_hash,
                config_hash,
                core_contract_client.implementation_address(),
                Address::from_str(&self.arg_config.verifier_address.clone()).unwrap(),
                false,
            )
            .await;
        core_contract_client
            .upgrade_to_core_contract(
                0u64.into(),
                0u64.into(),
                0u64.into(),
                program_hash,
                config_hash,
                core_contract_client.implementation_address(),
                Address::from_str(&self.arg_config.verifier_address.clone()).unwrap(),
                false,
            )
            .await;
        core_contract_client
            .register_operator_core_contract(Address::from_str(&self.arg_config.operator_address.clone()).unwrap())
            .await;
        core_contract_client
            .nominate_governor_core_contract(Address::from_str(&self.arg_config.l1_multisig_address.clone()).unwrap())
            .await;
        core_contract_client
            .nominate_governor_core_contract_proxy(
                Address::from_str(&self.arg_config.l1_multisig_address.clone()).unwrap(),
            )
            .await;

        CoreContractStarknetL1Output { core_contract_client }
    }
}
