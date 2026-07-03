import { Address, address } from "@solana/kit";
import { Keypair, PublicKey, SYSVAR_RENT_PUBKEY, TransactionInstruction } from "@solana/web3.js";
import { findMarketPda, findVaultPda } from "../clients/js/src/generated";

import { MPL_TOKEN_METADATA_PROGRAM_ID } from "@metaplex-foundation/mpl-token-metadata";
import { Program } from "@anchor-lang/core";
import { SYSTEM_PROGRAM_ID } from "@anchor-lang/core/dist/cjs/native/system";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { YukiWorldCup } from "../target/types/yuki_world_cup";

const METADATA_PROGRAM_ID = new PublicKey(MPL_TOKEN_METADATA_PROGRAM_ID.toString());

export async function createMarketIx(
  program: Program<YukiWorldCup>,
  authority: PublicKey,
  config: Address,
  mint: PublicKey,
): Promise<[TransactionInstruction, Keypair]> {
  const [market] = await findMarketPda({ mint: address(mint.toString()) });
  const [vault] = await findVaultPda({ mint: address(mint.toString()) });

  const receiptMint = Keypair.generate();

  const [metadataAccount] = PublicKey.findProgramAddressSync(
    [Buffer.from("metadata"), METADATA_PROGRAM_ID.toBuffer(), receiptMint.publicKey.toBuffer()],
    METADATA_PROGRAM_ID,
  );

  const ix = await program.methods
    .createMarket()
    .accountsStrict({
      authority,
      config,
      market: market,
      mint: mint,
      vault: vault,
      receiptMint: receiptMint.publicKey,
      metadataAccount: metadataAccount,
      tokenMetadataProgram: METADATA_PROGRAM_ID,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SYSTEM_PROGRAM_ID,
      rent: SYSVAR_RENT_PUBKEY,
    })
    .instruction();

  return [ix, receiptMint];
}
