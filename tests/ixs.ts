import { Address, address } from "@solana/kit";
import { Keypair, PublicKey, TransactionInstruction } from "@solana/web3.js";
import { findMarketPda, findVaultPda } from "../clients/js/src/generated";

import { Integral } from "../target/types/integral";
import { Program } from "@anchor-lang/core";
import { SYSTEM_PROGRAM_ID } from "@anchor-lang/core/dist/cjs/native/system";
import { TOKEN_2022_PROGRAM_ID } from "@solana/spl-token";

export async function createMarketIx(
  program: Program<Integral>,
  authority: PublicKey,
  config: Address,
  mint: PublicKey,
): Promise<[TransactionInstruction, Keypair, Address]> {
  const [market] = await findMarketPda({ mint: address(mint.toString()) });
  const [vault] = await findVaultPda({ mint: address(mint.toString()) });

  const receiptMint = Keypair.generate();

  const ix = await program.methods
    .createMarket()
    .accountsStrict({
      authority,
      config,
      market: market,
      mint: mint,
      vault: vault,
      receiptMint: receiptMint.publicKey,
      tokenProgram: TOKEN_2022_PROGRAM_ID,
      systemProgram: SYSTEM_PROGRAM_ID,
    })
    .instruction();

  return [ix, receiptMint, market];
}
