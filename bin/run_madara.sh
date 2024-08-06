#!/bin/bash

# This binary is build using the disabled fee flag on madara.

./madara setup --from-local ./configs --base-path=./setup_dir --chain=dev
RUST_LOG=runtime::starknet ./madara --dev --tx-ban-seconds=0 --pruning=archive --base-path=./setup_dir --settlement=Ethereum --settlement-conf eth-config.json --rpc-cors all