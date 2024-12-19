.PHONY: setup setup-linux ensure-asdf  starkgate-contracts-latest braavos-account-cairo argent-contracts-starknet artifacts starkgate-contracts-legacy

STARKGATE_CONTRACTS_VERSION_TAG="v2.0.1"
ARGENT_CONTRACTS_COMMIT_HASH="1352198956f36fb35fa544c4e46a3507a3ec20e3"
BRAAVOS_CONTRACTS_COMMIT_HASH="12b82a87b93ba9bfdf2cbbde2566437df2e0c6c8"

SHELL := /bin/bash
HOME_DIR := $(HOME)

VENV_DIR := $(PWD)/venv
VENV_BIN := $(VENV_DIR)/bin
PYTHON := $(VENV_BIN)/python
PIP := $(VENV_BIN)/pip
SOLC_SELECT := $(VENV_BIN)/solc-select
SOLC := $(VENV_BIN)/solc
# Add a target to create and setup virtual environment
setup-venv:
	python3 -m venv $(VENV_DIR)
	$(PIP) install --upgrade pip
	$(PIP) install solc-select
	$(SOLC_SELECT) install 0.8.24
	$(SOLC_SELECT) use 0.8.24
	$(VENV_BIN)/solc --version

ensure-asdf:
	@if [ "$$(uname)" = "Darwin" ] && command -v brew >/dev/null 2>&1; then \
		BREW_ASDF_PATH="$$(brew --prefix asdf)/libexec/asdf.sh"; \
		if [ ! -f "$$BREW_ASDF_PATH" ]; then \
			echo "Installing ASDF via Homebrew on macOS..."; \
			brew install asdf; \
		fi; \
		. "$$BREW_ASDF_PATH" && \
		if ! asdf plugin list | grep -q scarb; then \
			asdf plugin add scarb https://github.com/software-mansion/asdf-scarb.git; \
		fi; \
	else \
		if [ ! -f "$(HOME_DIR)/.asdf/asdf.sh" ]; then \
			echo "ASDF not found in $(HOME_DIR)/.asdf/"; \
			echo "Please install ASDF first:"; \
			echo "git clone https://github.com/asdf-vm/asdf.git ~/.asdf --branch v0.14.1"; \
			exit 1; \
		fi; \
		. "$(HOME_DIR)/.asdf/asdf.sh" && \
		if ! asdf plugin list | grep -q scarb; then \
			asdf plugin add scarb https://github.com/software-mansion/asdf-scarb.git; \
		fi; \
	fi

# Setup cairo for mac os
setup:
	mkdir -p .cairo
	cd .cairo && \
	wget -c https://github.com/starkware-libs/cairo/releases/download/v2.3.0/release-aarch64-apple-darwin.tar -O - | tar -xz

setup-linux:
	mkdir -p .cairo
	cd .cairo && \
  	wget -c https://github.com/starkware-libs/cairo/releases/download/v2.3.0/release-x86_64-unknown-linux-musl.tar.gz -O - | tar -xz

starkgate-contracts-latest: setup-venv
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
	$(SOLC_SELECT) install 0.8.24 && $(SOLC_SELECT) use 0.8.24 && \
	cd lib/starkgate-contracts-latest && \
	./scripts/setup.sh && \
	FILES=$$(cat src/solidity/files_to_compile.txt) && \
	$(SOLC) $$FILES --allow-paths .=., --optimize --optimize-runs 200 --overwrite --combined-json abi,bin -o artifacts && \
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


braavos-account-cairo: ensure-asdf
	# Building
	@if [ "$$(uname)" = "Darwin" ] && command -v brew >/dev/null 2>&1; then \
		. "$$(brew --prefix asdf)/libexec/asdf.sh" && \
		cd ./lib/braavos-account-cairo && \
		git checkout $(BRAAVOS_CONTRACTS_COMMIT_HASH) && \
		. "$$(brew --prefix asdf)/libexec/asdf.sh" && \
		asdf install scarb 2.8.4 && \
		. "$$(brew --prefix asdf)/libexec/asdf.sh" && \
		asdf local scarb 2.8.4 && \
		scarb build; \
	else \
		. "$(HOME_DIR)/.asdf/asdf.sh" && \
		cd ./lib/braavos-account-cairo && \
		git checkout $(BRAAVOS_CONTRACTS_COMMIT_HASH) && \
		. "$(HOME_DIR)/.asdf/asdf.sh" && \
		asdf install scarb 2.8.4 && \
		. "$(HOME_DIR)/.asdf/asdf.sh" && \
		asdf local scarb 2.8.4 && \
		scarb build; \
	fi
	# Copying Contracts
	cp ./lib/braavos-account-cairo/target/dev/braavos_account_BraavosAccount.contract_class.json ./artifacts/BraavosAccount.sierra.json
	cp ./lib/braavos-account-cairo/target/dev/braavos_account_BraavosAccount.compiled_contract_class.json ./artifacts/BraavosAccount.casm.json
	cp ./lib/braavos-account-cairo/target/dev/braavos_account_BraavosBaseAccount.contract_class.json ./artifacts/BraavosBaseAccount.sierra.json
	cp ./lib/braavos-account-cairo/target/dev/braavos_account_BraavosBaseAccount.compiled_contract_class.json ./artifacts/BraavosBaseAccount.casm.json


argent-contracts-starknet: ensure-asdf
	# Building
	@if [ "$$(uname)" = "Darwin" ] && command -v brew >/dev/null 2>&1; then \
		. "$$(brew --prefix asdf)/libexec/asdf.sh" && \
		cd ./lib/argent-contracts-starknet && \
		git checkout $(ARGENT_CONTRACTS_COMMIT_HASH) && \
		asdf install scarb 2.6.3 && \
		asdf local scarb 2.6.3 && \
		scarb build; \
	else \
		. "$(HOME_DIR)/.asdf/asdf.sh" && \
		cd ./lib/argent-contracts-starknet && \
		git checkout $(ARGENT_CONTRACTS_COMMIT_HASH) && \
		asdf install scarb 2.6.3 && \
		asdf local scarb 2.6.3 && \
		scarb build; \
	fi
	# Copying Contracts
	cp ./lib/argent-contracts-starknet/target/dev/argent_ArgentAccount.contract_class.json ./artifacts/ArgentAccount.sierra.json
	cp ./lib/argent-contracts-starknet/target/dev/argent_ArgentAccount.compiled_contract_class.json ./artifacts/ArgentAccount.casm.json

build-contracts:
	make starkgate-contracts-legacy
	make starkgate-contracts-latest
	make braavos-account-cairo
	make argent-contracts-starknet

artifacts-linux:
	make setup-linux
	make build-contracts

artifacts:
	make setup
	make build-contracts