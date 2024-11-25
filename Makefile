.PHONY: setup setup-linux starkgate-contracts-latest starkgate-contracts-old braavos-account-cairo argent-contracts-starknet artifacts

STARKGATE_CONTRACTS_COMMIT_HASH="45941888479663ac93e898cd7f8504fa9066c54c"
ARGENT_CONTRACTS_COMMIT_HASH="1352198956f36fb35fa544c4e46a3507a3ec20e3"
BRAAVOS_CONTRACTS_COMMIT_HASH="12b82a87b93ba9bfdf2cbbde2566437df2e0c6c8"

# Setup cairo for mac os
setup:
	mkdir -p .cairo
	cd .cairo && \
	wget -c https://github.com/starkware-libs/cairo/releases/download/v2.6.3/release-aarch64-apple-darwin.tar -O - | tar -xz

setup-linux:
	mkdir -p .cairo
	cd .cairo && \
  	wget -c https://github.com/starkware-libs/cairo/releases/download/v2.6.3/release-x86_64-unknown-linux-musl.tar.gz -O - | tar -xz

starkgate-contracts-latest:
	# Building
	cd lib/starkgate-contracts-latest && \
 	git checkout $(STARKGATE_CONTRACTS_COMMIT_HASH) && \
 	rm -rf starkware && \
 	tar xvf .dep/starkware-solidity-dependencies.tar && \
 	mkdir -p cairo_contracts && \
	../../.cairo/cairo/bin/starknet-compile src  --contract-path src::strk::erc20_lockable::ERC20Lockable cairo_contracts/ERC20Lockable.sierra && \
	../../.cairo/cairo/bin/starknet-compile src  --contract-path src::token_bridge::TokenBridge cairo_contracts/TokenBridge.sierra && \
	../../.cairo/cairo/bin/starknet-compile src  --contract-path openzeppelin::token::erc20_v070::erc20::ERC20 cairo_contracts/ERC20.sierra

	./.cairo/cairo/bin/starknet-sierra-compile ./lib/starkgate-contracts-latest/cairo_contracts/ERC20Lockable.sierra ./lib/starkgate-contracts-latest/cairo_contracts/ERC20Lockable.casm
	./.cairo/cairo/bin/starknet-sierra-compile ./lib/starkgate-contracts-latest/cairo_contracts/TokenBridge.sierra ./lib/starkgate-contracts-latest/cairo_contracts/TokenBridge.casm
	# Copying Contracts
	mkdir -p artifacts
	cp ./lib/starkgate-contracts-latest/cairo_contracts/ERC20Lockable.sierra ./artifacts/erc20.sierra.json
	cp ./lib/starkgate-contracts-latest/cairo_contracts/ERC20Lockable.casm ./artifacts/erc20.casm.json
	cp ./lib/starkgate-contracts-latest/cairo_contracts/TokenBridge.sierra ./artifacts/token_bridge.sierra.json
	cp ./lib/starkgate-contracts-latest/cairo_contracts/TokenBridge.casm ./artifacts/token_bridge.casm.json

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

make artifacts-linux:
	make setup-linux
	make starkgate-contracts-latest
	make braavos-account-cairo
	make argent-contracts-starknet

make artifacts:
	make setup
	make starkgate-contracts-latest
	make braavos-account-cairo
	make argent-contracts-starknet