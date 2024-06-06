// import { ApiPromise, HttpProvider } from "@polkadot/api";
// import * as dotenv from "dotenv";
// dotenv.config();
//
// const VAL = process.argv[2];
//
// const main = async (option: boolean) => {
//   const SEQ_RPC_URL: string = process.env.ROLLUP_SEQ_URL || "";
//   const polkadotProvider = new HttpProvider(SEQ_RPC_URL);
//
//   const api = await ApiPromise.create({ provider: polkadotProvider });
//   const extrinsic = api.tx.starknet.setDisableFee(option);
//   await extrinsic.send();
//
//   // sleeping for some time
//   await new Promise((resolve) => setTimeout(resolve, 7000));
// };
//
// main(VAL as unknown as boolean).then((r) =>
//   console.log("Script Executed Successfully : )")
// );

import {RpcProvider, stark, hash} from "starknet";
import {writeFileSync} from "node:fs";

const main = async () => {

    const sepoliaProvider = new RpcProvider({
        nodeUrl:
            'https://starknet-sepolia.infura.io/v3/6cb41f0e9a564ef18240ff6bf1a7427f',
    });

    const class_fetch = await sepoliaProvider.getClassByHash("0x5c478ee27f2112411f86f207605b2e2c58cdb647bac0df27f660ef2252359c6");

    // console.log("type of class_fetch : ", class_fetch);

    const program = class_fetch.program;

    const decompress_program = stark.decompressProgram(program);
    // console.log(decompress_program);

    // console.log("entrypoints : ", class_fetch.entry_points_by_type);

    const abi_object = {
        abi : class_fetch.abi,
        entry_points_by_type: class_fetch.entry_points_by_type,
        program: decompress_program
    }

    // writeFileSync("./temp.json", abi_object, 'utf8');

    const parsed = JSON.stringify(abi_object, (key, value) =>
        typeof value === 'bigint'
            ? value.toString()
            : value // return everything else unchanged
    )

    console.log(parsed)

    const class_hash = hash.computeLegacyContractClassHash(abi_object);
    const class_hash_parsed = hash.computeLegacyContractClassHash(parsed);

    console.log("class hash : ", class_hash);
    console.log("class hash parsed : ", class_hash_parsed);
}

main()
