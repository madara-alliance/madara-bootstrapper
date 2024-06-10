import {
  RpcProvider,
  hash,
  Signer,
  Account,
  Contract,
  CallData,
} from "starknet";
import ABI from "../src/contracts/OpenZeppelinAccount.json" with { type: "json" };

const main = async () => {
  const provider = new RpcProvider({
    nodeUrl: "https://1a8d-2405-201-4059-e00f-f3-11c5-1e18-345.ngrok-free.app",
  });

  const signer = new Signer("0xabcd");

  const account = new Account(
    provider,
    "0x2fcd7dc99066d2328f5b32cb50dfe8e1815d412551ab8c70519f86c8b7e6838",
    signer,
  );

  const contract = new Contract(
    ABI.abi,
    "0x2fcd7dc99066d2328f5b32cb50dfe8e1815d412551ab8c70519f86c8b7e6838",
    account,
  );

  const calldata = CallData.toCalldata([
    "0xd0e183745e9dae3e4e78a8ffedcce0903fc4900beace4e0abf192d4c202da3",
    "0x322c2610264639f6b2cee681ac53fa65c37e187ea24292d1b21d859c55e1a78",
    "0x0000000000000000000000000000000000000000000000000000000000000001",
    "0x0000000000000000000000000000000000000000000000000000000000000000",
    "0x0000000000000000000000000000000000000000000000000000000000000001",
  ]);

  const call = await contract.invoke("deploy_contract", calldata, {
    maxFee: "10000000000000000000000",
  });

  const calculate_address = hash.calculateContractAddressFromHash(
    "0x322c2610264639f6b2cee681ac53fa65c37e187ea24292d1b21d859c55e1a78",
    "0x00d0e183745e9dae3e4e78a8ffedcce0903fc4900beace4e0abf192d4c202da3",
    calldata,
    "0x2fcd7dc99066d2328f5b32cb50dfe8e1815d412551ab8c70519f86c8b7e6838",
  );

  console.log("call result : ", call);
  console.log("calculate address : ", calculate_address);
};

async function eth_address() {
  const calculate_address = hash.calculateContractAddressFromHash(
    "0x322c2610264639f6b2cee681ac53fa65c37e187ea24292d1b21d859c55e1a78",
    "0x00d0e183745e9dae3e4e78a8ffedcce0903fc4900beace4e0abf192d4c202da3",
    ["0"],
    "1",
  );
  console.log("this is the address - ", calculate_address);
}

// main();
eth_address();
