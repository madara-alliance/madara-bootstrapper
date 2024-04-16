#!/bin/bash

echo ">>>> Setting up deps for starkgate contracts 2.0.1 release"
./starkgate-contracts-2-0-1/scripts/setup.sh

echo ">>>> Building starkgate 2.0.1 contracts"
./starkgate-contracts-2-0-1/scripts/build-cairo.sh
./starkgate-contracts-2-0-1/scripts/build-solidity.sh
./starkgate-contracts-2-0-1/.downloads/cairo/bin/starknet-sierra-compile ./starkgate-contracts-2-0-1/cairo_contracts/ERC20.sierra ./starkgate-contracts-2-0-1/cairo_contracts/ERC20.casm
./starkgate-contracts-2-0-1/.downloads/cairo/bin/starknet-sierra-compile ./starkgate-contracts-2-0-1/cairo_contracts/TokenBridge.sierra ./starkgate-contracts-2-0-1/cairo_contracts/TokenBridge.casm

echo ">>>> copying the built contracts 2.0.1"
cp ./starkgate-contracts-2-0-1/cairo_contracts/ERC20.sierra ./src/contracts/erc20.sierra.json
cp ./starkgate-contracts-2-0-1/cairo_contracts/ERC20.casm ./src/contracts/erc20.casm.json
cp ./starkgate-contracts-2-0-1/cairo_contracts/TokenBridge.sierra ./src/contracts/token_bridge.sierra.json
cp ./starkgate-contracts-2-0-1/cairo_contracts/TokenBridge.casm ./src/contracts/token_bridge.casm.json

#echo ">>>> Setting up deps for starkgate contracts 0.9.0 release"
#cd starkgate-contracts || return
#./build.sh
#cd ..
