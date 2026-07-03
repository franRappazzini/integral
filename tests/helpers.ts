import * as anchor from "@anchor-lang/core";

import { getConfigDecoder } from "../clients/js/src/generated";

export async function getConfigAccount(connection: anchor.web3.Connection, config: anchor.Address) {
  // const [config] = await findConfigPda();
  const data = (await connection.getAccountInfo(new anchor.web3.PublicKey(config.toString())))
    ?.data;
  return data ? getConfigDecoder().decode(data) : data;
}
