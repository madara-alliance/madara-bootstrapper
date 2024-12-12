.PHONY: setup setup-linux starkgate-contracts-latest braavos-account-cairo argent-contracts-starknet artifacts starkgate-contracts-legacy

STARKGATE_CONTRACTS_VERSION_TAG="v2.0.1"
ARGENT_CONTRACTS_COMMIT_HASH="1352198956f36fb35fa544c4e46a3507a3ec20e3"
BRAAVOS_CONTRACTS_COMMIT_HASH="12b82a87b93ba9bfdf2cbbde2566437df2e0c6c8"

# Setup cairo for mac os
setup:
	mkdir -p .cairo
	cd .cairo && \
	wget -c https://github.com/starkware-libs/cairo/releases/download/v2.3.0/release-aarch64-apple-darwin.tar -O - | tar -xz

setup-linux:
	mkdir -p .cairo
	cd .cairo && \
  	wget -c https://github.com/starkware-libs/cairo/releases/download/v2.3.0/release-x86_64-unknown-linux-musl.tar.gz -O - | tar -xz

starkgate-contracts-latest:
	# Building L2 contracts
	# =====================
	cd lib/starkgate-contracts-latest && \
 	git checkout $(STARKGATE_CONTRACTS_VERSION_TAG) && \
 	rm -rf starkware && \
 	tar xvf .dep/starkware-solidity-dependencies.tar && \
 	mkdir -p cairo_contracts && \
	../../.cairo/cairo/bin/starknet-compile src  --contract-path src::strk::erc20_lockable::ERC20Lockable cairo_contracts/ERC20Lockable.sierra && \
	../../.cairo/cairo/bin/starknet-compile src  --contract-path src::token_bridge::TokenBridge cairo_contracts/TokenBridge.sierra && \
	../../.cairo/cairo/bin/starknet-compile src  --contract-path openzeppelin::token::erc20_v070::erc20::ERC20 cairo_contracts/ERC20.sierra && \
	../../.cairo/cairo/bin/starknet-compile src  --contract-path src::legacy_bridge_eic::LegacyBridgeUpgradeEIC cairo_contracts/LegacyBridgeUpgradeEIC.sierra && \
	../../.cairo/cairo/bin/starknet-compile src  --contract-path src::roles_init_eic::RolesExternalInitializer cairo_contracts/RolesExternalInitializer.sierra
	# Compiling Casm
	./.cairo/cairo/bin/starknet-sierra-compile ./lib/starkgate-contracts-latest/cairo_contracts/ERC20Lockable.sierra ./lib/starkgate-contracts-latest/cairo_contracts/ERC20Lockable.casm
	./.cairo/cairo/bin/starknet-sierra-compile ./lib/starkgate-contracts-latest/cairo_contracts/TokenBridge.sierra ./lib/starkgate-contracts-latest/cairo_contracts/TokenBridge.casm
	./.cairo/cairo/bin/starknet-sierra-compile ./lib/starkgate-contracts-latest/cairo_contracts/LegacyBridgeUpgradeEIC.sierra ./lib/starkgate-contracts-latest/cairo_contracts/LegacyBridgeUpgradeEIC.casm
	./.cairo/cairo/bin/starknet-sierra-compile ./lib/starkgate-contracts-latest/cairo_contracts/RolesExternalInitializer.sierra ./lib/starkgate-contracts-latest/cairo_contracts/RolesExternalInitializer.casm
	./.cairo/cairo/bin/starknet-sierra-compile ./lib/starkgate-contracts-latest/cairo_contracts/ERC20.sierra ./lib/starkgate-contracts-latest/cairo_contracts/ERC20.casm
	# Copying Contracts
	mkdir -p artifacts
	cp ./lib/starkgate-contracts-latest/cairo_contracts/ERC20Lockable.sierra ./artifacts/erc20.sierra.json
	cp ./lib/starkgate-contracts-latest/cairo_contracts/ERC20Lockable.casm ./artifacts/erc20.casm.json
	cp ./lib/starkgate-contracts-latest/cairo_contracts/TokenBridge.sierra ./artifacts/token_bridge.sierra.json
	cp ./lib/starkgate-contracts-latest/cairo_contracts/TokenBridge.casm ./artifacts/token_bridge.casm.json
	cp ./lib/starkgate-contracts-latest/cairo_contracts/LegacyBridgeUpgradeEIC.sierra ./artifacts/token_bridge_eic.sierra.json
	cp ./lib/starkgate-contracts-latest/cairo_contracts/LegacyBridgeUpgradeEIC.casm ./artifacts/token_bridge_eic.casm.json
	cp ./lib/starkgate-contracts-latest/cairo_contracts/RolesExternalInitializer.sierra ./artifacts/eth_token_eic.sierra.json
	cp ./lib/starkgate-contracts-latest/cairo_contracts/RolesExternalInitializer.casm ./artifacts/eth_token_eic.casm.json
	cp ./lib/starkgate-contracts-latest/cairo_contracts/ERC20.sierra ./artifacts/ERC20_070.sierra.json
	cp ./lib/starkgate-contracts-latest/cairo_contracts/ERC20.casm ./artifacts/ERC20_070.casm.json
	# Building L1 contracts
	# =====================
	# Configure solidity version
	solc-select install 0.8.24 && solc-select use 0.8.24
	# Building
	cd lib/starkgate-contracts-latest && \
	./scripts/setup.sh && \
	FILES=$$(cat src/solidity/files_to_compile.txt) && \
	solc $$FILES --allow-paths .=., --optimize --optimize-runs 200 --overwrite --combined-json abi,bin -o artifacts && \
	./scripts/extract_artifacts.py
	# Copying Contracts
	mkdir -p artifacts/upgrade-contracts
	cp lib/starkgate-contracts-latest/artifacts/StarknetEthBridge.json artifacts/upgrade-contracts/eth_bridge_upgraded.json
	cp lib/starkgate-contracts-latest/artifacts/StarkgateUpgradeAssistExternalInitializer.json artifacts/upgrade-contracts/eic_eth_bridge.json

starkgate-contracts-legacy:
	# Building Contracts
	rm -rf lib/starkgate-contracts-old/Dockerfile
	cp ./build-artifacts/Dockerfile ./lib/starkgate-contracts-old/Dockerfile
	cd lib/starkgate-contracts-old && \
	docker build -t starkgate-build . && \
	mkdir -p starkgate-artifacts && \
	docker run -v ./starkgate-artifacts/:/mnt starkgate-build
	# Copying Contracts
	mkdir -p artifacts
	cp ./lib/starkgate-contracts-old/starkgate-artifacts/starkware/starknet/apps/starkgate/artifacts/cairo/token_bridge_1.json ./artifacts/legacy_token_bridge.json
	cp ./lib/starkgate-contracts-old/starkgate-artifacts/starkware/starknet/std_contracts/upgradability_proxy/proxy.json ./artifacts/proxy_starkgate.json
	cp ./lib/starkgate-contracts-old/starkgate-artifacts/starkware/starknet/std_contracts/ERC20/ERC20.json ./artifacts/ERC20.json

braavos-account-cairo:
	# Building
	asdf install scarb 2.8.4 && asdf global scarb 2.8.4
	cd ./lib/braavos-account-cairo && \
 	git checkout $(BRAAVOS_CONTRACTS_COMMIT_HASH) && \
 	~/.asdf/installs/scarb/2.8.4/bin/scarb build
	# Copying Contracts
	cp ./lib/braavos-account-cairo/target/dev/braavos_account_BraavosAccount.contract_class.json ./artifacts/BraavosAccount.sierra.json
	cp ./lib/braavos-account-cairo/target/dev/braavos_account_BraavosAccount.compiled_contract_class.json ./artifacts/BraavosAccount.casm.json
	cp ./lib/braavos-account-cairo/target/dev/braavos_account_BraavosBaseAccount.contract_class.json ./artifacts/BraavosBaseAccount.sierra.json
	cp ./lib/braavos-account-cairo/target/dev/braavos_account_BraavosBaseAccount.compiled_contract_class.json ./artifacts/BraavosBaseAccount.casm.json

argent-contracts-starknet:
	# Building
	asdf install scarb 2.6.3 && asdf global scarb 2.6.3
	cd ./lib/argent-contracts-starknet && \
 	git checkout $(ARGENT_CONTRACTS_COMMIT_HASH) && \
 	~/.asdf/installs/scarb/2.6.3/bin/scarb build
	# Copying Contracts
	cp ./lib/argent-contracts-starknet/target/dev/argent_ArgentAccount.contract_class.json ./artifacts/ArgentAccount.sierra.json
	cp ./lib/argent-contracts-starknet/target/dev/argent_ArgentAccount.compiled_contract_class.json ./artifacts/ArgentAccount.casm.json

make build-contracts:
	make starkgate-contracts-legacy
	make starkgate-contracts-latest
	make braavos-account-cairo
	make argent-contracts-starknet

make artifacts-linux:
	make setup-linux
	make build-contracts

make artifacts:
	make setup
	make build-contracts