## Transactions from genesis

These are the steps used by starknet to setup the network on sepolia testnet.

Block 0
// DISABLE FEE

1. Declare an account class. This has the deploy_contract function (0x05c478ee27f2112411f86f207605b2e2c58cdb647bac0df27f660ef2252359c6)
2. Declare the the proxy (0x00d0e183745e9dae3e4e78a8ffedcce0903fc4900beace4e0abf192d4c202da3)
3. Deploy account in (1)
   1. Public key - 0x12c4df40394d06f157edec8d0e64db61fe0c271149ea860c8fe98def29ecf02
   2. Address - 0x043abaa073c768ebf039c0c4f46db9acc39e9ec165690418060a652aab39e7d8
4. Deploys proxy for ETH token -
   1. Address - 0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7
   2. Class hash - 0xd0e183745e9dae3e4e78a8ffedcce0903fc4900beace4e0abf192d4c202da3
5. Deploys proxy for Eth bridge
   1. Address - 0x04c5772d1914fe6ce891b64eb35bf3522aeae1315647314aac58b01137607f3f
   2. Class hash - 0x00d0e183745e9dae3e4e78a8ffedcce0903fc4900beace4e0abf192d4c202da3
6. Call init_governance on ETH token
   1. Code link - <https://github.com/starknet-io/starkgate-contracts/blob/82e651f5f642508577b6310f47d0d457f5f9a9bb/src/starkware/starknet/std_contracts/upgradability_proxy/governance.cairo#L49>
   2. Makes the caller a governor
7. Call init_governance on ETH bridge
   1. Same as above

Block 1

1. Declare ERC20 - 0x01b661756bf7d16210fc611626e1af4569baa1781ffc964bd018f4585ae241c1

Block 2

1. Declare Token bridge - 0x04f23a756b221f8ce46b72e6a6b10ee7ee6cf3b59790e76e02433104f9a8c5d1

Block 3

1. Deploy ERC20 - 0x023be95f90bf41685e18a4356e57b0cfdc1da22bf382ead8b64108353915c1e5
2. Deploy token bridge - 0x06d8ff7b212b08760c82e4a8f354f6ebc69d748290fa38e92eb859726a88f379

Block 4

1. Add implementation to ETH token proxy (deployed in block 0 txn 4) for ERC20 deployed in block 3 txn 1
2. Call upgrade_to on ETH token proxy to bring new implementation in effect immediately
3. Add implementation to ETH bridge proxy (deployed in block 0 txn 5) to token bridge deployed in block 3 txn 2
4. Upgrade to for above

Block 5

1. Call set_l1_bridge on Eth bridge proxy - StarknetEthBridge.sol address - 0x8453fc6cd1bcfe8d4dfc069c400b433054d47bdc
2. Call set_l2_token on Eth bridge proxy, Eth token address - 0x49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7

// ENABLE FEES
Block 6

1. Normal network continues

Block 7

1. Declare UDC

## Ethereum Sepolia

1. Deploy Proxy.sol
2. Deploy StarknetEthBridge.sol
3. Add implementation of Eth bridge in proxy
4. Upgrade implementation to eth bridge
5. Set max_deposit - 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
6. Set max_total_balance - 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
7. Set l2 token bridge - 0x04c5772d1914fe6ce891b64eb35bf3522aeae1315647314aac58b01137607f3f (Starkgate eth bridge proxy)
8. Deposit continues
