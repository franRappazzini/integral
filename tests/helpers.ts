import * as anchor from "@anchor-lang/core";

import { getConfigDecoder, getMarketDecoder } from "../clients/js/src/generated";

export async function getConfigAccount(connection: anchor.web3.Connection, config: anchor.Address) {
  const data = (await connection.getAccountInfo(new anchor.web3.PublicKey(config.toString())))
    ?.data;
  return data ? getConfigDecoder().decode(data) : data;
}

export async function getMarketAccount(connection: anchor.web3.Connection, market: anchor.Address) {
  const data = (await connection.getAccountInfo(new anchor.web3.PublicKey(market.toString())))
    ?.data;
  return data ? getMarketDecoder().decode(data) : data;
}
