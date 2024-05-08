import { ApiPromise, HttpProvider } from "@polkadot/api";
import * as dotenv from "dotenv";
dotenv.config();

const VAL = process.argv[2];

const main = async (option: boolean) => {
  const SEQ_RPC_URL: string = process.env.ROLLUP_SEQ_URL || "";
  const polkadotProvider = new HttpProvider(SEQ_RPC_URL);

  const api = await ApiPromise.create({ provider: polkadotProvider });
  const extrinsic = api.tx.starknet.setDisableFee(option);
  await extrinsic.send();

  // sleeping for some time
  await new Promise((resolve) => setTimeout(resolve, 7000));
};

main(VAL as unknown as boolean).then((r) =>
  console.log("Script Executed Successfully : )")
);
