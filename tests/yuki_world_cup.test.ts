import * as anchor from "@anchor-lang/core";

import {
  Account,
  TOKEN_PROGRAM_ID,
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
} from "@solana/spl-token";
import { LAMPORTS_PER_SOL, Transaction, sendAndConfirmTransaction } from "@solana/web3.js";
import {
  findConfigPda,
  findFarmerPositionPda,
  findMarketPda,
  findRewardVaultPda,
} from "../clients/js/src/generated";
import { getConfigAccount, getFarmerPositionAccount, getMarketAccount } from "./helpers";

import { Program } from "@anchor-lang/core";
import { SYSTEM_PROGRAM_ID } from "@anchor-lang/core/dist/cjs/native/system";
import { YukiWorldCup } from "../target/types/yuki_world_cup";
import { address } from "@solana/kit";
import { bn } from "./utils";
import { createMarketIx } from "./ixs";
import { expect } from "chai";

describe("yuki_world_cup", () => {
  const provider = anchor.AnchorProvider.env();
  const { connection, wallet } = provider;
  const payer = wallet.payer as anchor.web3.Keypair;

  anchor.setProvider(provider);

  const program = anchor.workspace.yukiWorldCup as Program<YukiWorldCup>;

  let authorityAta: Account;
  let farmerArgAta: Account;

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
    farmerArgAta = await getOrCreateAssociatedTokenAccount(
      connection,
      payer,
      argMint,
      wallet.publicKey,
    );

    // reward mint
    await mintTo(
      connection,
      payer,
      rewardMint,
      authorityAta.address,
      wallet.publicKey,
      LAMPORTS_PER_SOL,
    );
    // arg mint
    await mintTo(
      connection,
      payer,
      argMint,
      farmerArgAta.address,
      wallet.publicKey,
      LAMPORTS_PER_SOL,
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

    const [argIx, receiptMintArg, marketArg] = await createMarketIx(
      program,
      wallet.publicKey,
      config,
      argMint,
    );
    const [fraIx, receiptMintFra, marketFra] = await createMarketIx(
      program,
      wallet.publicKey,
      config,
      fraMint,
    );
    const [spaIx, receiptMintSpa, marketSpa] = await createMarketIx(
      program,
      wallet.publicKey,
      config,
      spaMint,
    );

    const tx = new Transaction().add(argIx, fraIx, spaIx);
    tx.feePayer = wallet.publicKey;
    tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;

    const sig = await sendAndConfirmTransaction(connection, tx, [
      payer,
      receiptMintArg, // receipt mint creation
      receiptMintFra, // receipt mint creation
      receiptMintSpa, // receipt mint creation
    ]);

    console.log("create_market tx signature:", sig);

    const marketArgAccount = await getMarketAccount(connection, marketArg);
    const marketFraAccount = await getMarketAccount(connection, marketFra);
    const marketSpaAccount = await getMarketAccount(connection, marketSpa);

    expect(marketArgAccount).exist;
    expect(marketFraAccount).exist;
    expect(marketSpaAccount).exist;
  });

  it("`deposit` ix", async () => {
    const AMOUNT = LAMPORTS_PER_SOL / 2;

    const tx = await program.methods
      .deposit(bn(AMOUNT))
      .accounts({
        farmer: wallet.publicKey,
        mint: argMint,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    console.log("deposit tx signature:", tx);

    // checks
    const [market] = await findMarketPda({ mint: address(argMint.toString()) });
    const [farmerPosition] = await findFarmerPositionPda({
      farmer: address(wallet.publicKey.toString()),
      market,
    });

    const marketAccount = await getMarketAccount(connection, market);
    const farmerPositionAccount = await getFarmerPositionAccount(connection, farmerPosition);

    expect(Number(marketAccount?.totalDeposited)).eq(AMOUNT);

    expect(farmerPositionAccount).exist;
    expect(Number(farmerPositionAccount?.amount)).eq(AMOUNT);
    expect(farmerPositionAccount?.isInitialized).to.be.true;
  });
});
