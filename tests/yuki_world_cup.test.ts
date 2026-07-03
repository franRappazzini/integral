import * as anchor from "@anchor-lang/core";

import {
  Account,
  TOKEN_PROGRAM_ID,
  createMint,
  getOrCreateAssociatedTokenAccount,
} from "@solana/spl-token";
import { Transaction, sendAndConfirmTransaction } from "@solana/web3.js";
import {
  findConfigPda,
  findMarketPda,
  findRewardVaultPda,
  findVaultPda,
} from "../clients/js/src/generated";

import { Program } from "@anchor-lang/core";
import { SYSTEM_PROGRAM_ID } from "@anchor-lang/core/dist/cjs/native/system";
import { YukiWorldCup } from "../target/types/yuki_world_cup";
import { address } from "@solana/kit";
import { bn } from "./utils";
import { expect } from "chai";
import { getConfigAccount } from "./helpers";

describe("yuki_world_cup", () => {
  const provider = anchor.AnchorProvider.env();
  const { connection, wallet } = provider;
  const payer = wallet.payer as anchor.web3.Keypair;

  anchor.setProvider(provider);

  const program = anchor.workspace.yukiWorldCup as Program<YukiWorldCup>;

  let authorityAta: Account;

  let rewardMint: anchor.web3.PublicKey;
  let argMint: anchor.web3.PublicKey;
  let fraMint: anchor.web3.PublicKey;
  let spaMint: anchor.web3.PublicKey;

  // mint and ata creation
  before(async () => {
    rewardMint = await createMint(connection, payer, wallet.publicKey, null, 6);
    argMint = await createMint(connection, payer, wallet.publicKey, null, 6);
    fraMint = await createMint(connection, payer, wallet.publicKey, null, 6);
    spaMint = await createMint(connection, payer, wallet.publicKey, null, 6);

    authorityAta = await getOrCreateAssociatedTokenAccount(
      connection,
      payer,
      rewardMint,
      wallet.publicKey,
    );
  });

  it("`initialize` ix", async () => {
    const REWARD_AMOUNT = 1000_000_000; // 1000 usdc/cash
    const FEE_BPS = 500; // 0.5% fee

    const [config] = await findConfigPda();
    const [rewardVault] = await findRewardVaultPda({ rewardMint: address(rewardMint.toString()) });

    const tx = await program.methods
      .initialize(bn(REWARD_AMOUNT), FEE_BPS)
      .accountsStrict({
        authority: wallet.publicKey,
        config,
        rewardMint,
        authorityAta: authorityAta.address,
        rewardVault,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SYSTEM_PROGRAM_ID,
      })
      .rpc();

    console.log("initialize tx signature:", tx);

    const configAccount = await getConfigAccount(connection, config);
    expect(configAccount).exist;
  });

  it("`create_market` ix", async () => {
    const [config] = await findConfigPda();
    const [marketArg] = await findMarketPda({ mint: address(argMint.toString()) });
    const [marketFra] = await findMarketPda({ mint: address(fraMint.toString()) });
    const [marketSpa] = await findMarketPda({ mint: address(spaMint.toString()) });
    const [vaultArg] = await findVaultPda({ mint: address(argMint.toString()) });
    const [vaultFra] = await findVaultPda({ mint: address(fraMint.toString()) });
    const [vaultSpa] = await findVaultPda({ mint: address(spaMint.toString()) });

    const argIx = await program.methods
      .createMarket()
      .accountsStrict({
        authority: wallet.publicKey,
        config,
        market: marketArg,
        mint: argMint,
        vault: vaultArg,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SYSTEM_PROGRAM_ID,
      })
      .instruction();

    const fraIx = await program.methods
      .createMarket()
      .accountsStrict({
        authority: wallet.publicKey,
        config,
        market: marketFra,
        mint: fraMint,
        vault: vaultFra,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SYSTEM_PROGRAM_ID,
      })
      .instruction();

    const spaIx = await program.methods
      .createMarket()
      .accountsStrict({
        authority: wallet.publicKey,
        config,
        market: marketSpa,
        mint: spaMint,
        vault: vaultSpa,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SYSTEM_PROGRAM_ID,
      })
      .instruction();

    const tx = new Transaction().add(argIx, fraIx, spaIx);
    tx.feePayer = wallet.publicKey;
    tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;

    const sig = await sendAndConfirmTransaction(connection, tx, [payer]);
    console.log("create_market tx signature:", sig);
  });
});
