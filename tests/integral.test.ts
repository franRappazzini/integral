import * as anchor from "@anchor-lang/core";

import {
  Account,
  TOKEN_2022_PROGRAM_ID,
  createCloseAccountInstruction,
  createMint,
  getAssociatedTokenAddressSync,
  getOrCreateAssociatedTokenAccount,
  mintTo,
} from "@solana/spl-token";
import {
  LAMPORTS_PER_SOL,
  PublicKey,
  Transaction,
  sendAndConfirmTransaction,
} from "@solana/web3.js";
import {
  MarketStatus,
  findConfigPda,
  findMarketPda,
  findRewardVaultPda,
} from "../clients/js/src/generated";
import { getConfigAccount, getMarketAccount } from "./helpers";

import { Integral } from "../target/types/integral";
import { Program } from "@anchor-lang/core";
import { SYSTEM_PROGRAM_ID } from "@anchor-lang/core/dist/cjs/native/system";
import { address } from "@solana/kit";
import { bn } from "./utils";
import { createMarketIx } from "./ixs";
import { expect } from "chai";
import { simulateTransaction } from "@anchor-lang/core/dist/cjs/utils/rpc";

describe("integral", () => {
  const provider = anchor.AnchorProvider.env();
  const { connection, wallet } = provider;
  const payer = wallet.payer as anchor.web3.Keypair;

  anchor.setProvider(provider);

  const program = anchor.workspace.integral as Program<Integral>;

  let authorityAta: Account;
  let farmerArgAta: Account;

  let rewardMint: anchor.web3.PublicKey;
  let argMint: anchor.web3.PublicKey;
  let fraMint: anchor.web3.PublicKey;
  let spaMint: anchor.web3.PublicKey;

  // mint and ata creation
  before(async () => {
    rewardMint = await createMint(
      connection,
      payer,
      wallet.publicKey,
      null,
      6,
      undefined,
      undefined,
      TOKEN_2022_PROGRAM_ID,
    );
    argMint = await createMint(
      connection,
      payer,
      wallet.publicKey,
      null,
      6,
      undefined,
      undefined,
      TOKEN_2022_PROGRAM_ID,
    );
    fraMint = await createMint(
      connection,
      payer,
      wallet.publicKey,
      null,
      6,
      undefined,
      undefined,
      TOKEN_2022_PROGRAM_ID,
    );
    spaMint = await createMint(
      connection,
      payer,
      wallet.publicKey,
      null,
      6,
      undefined,
      undefined,
      TOKEN_2022_PROGRAM_ID,
    );

    authorityAta = await getOrCreateAssociatedTokenAccount(
      connection,
      payer,
      rewardMint,
      wallet.publicKey,
      false,
      undefined,
      undefined,
      TOKEN_2022_PROGRAM_ID,
    );
    farmerArgAta = await getOrCreateAssociatedTokenAccount(
      connection,
      payer,
      argMint,
      wallet.publicKey,
      false,
      undefined,
      undefined,
      TOKEN_2022_PROGRAM_ID,
    );

    // reward mint
    await mintTo(
      connection,
      payer,
      rewardMint,
      authorityAta.address,
      wallet.publicKey,
      LAMPORTS_PER_SOL * 2,
      undefined,
      undefined,
      TOKEN_2022_PROGRAM_ID,
    );
    // arg mint
    await mintTo(
      connection,
      payer,
      argMint,
      farmerArgAta.address,
      wallet.publicKey,
      LAMPORTS_PER_SOL,
      undefined,
      undefined,
      TOKEN_2022_PROGRAM_ID,
    );
  });

  it("`initialize` ix", async () => {
    const REWARD_AMOUNT = 1000_000_000; // 1000 usdc/cash
    const FEE_BPS = 100; // 1% fee

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
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        systemProgram: SYSTEM_PROGRAM_ID,
      })
      .rpc();

    console.log("initialize tx signature:", tx);

    const configAccount = await getConfigAccount(connection, config);

    expect(configAccount).exist;
    expect(Number(configAccount?.rewardAmount)).eq(REWARD_AMOUNT);
  });

  it("`add_rewards` ix", async () => {
    const [config] = await findConfigPda();
    const preConfigAccount = await getConfigAccount(connection, config);
    const preAmount = Number(preConfigAccount?.rewardAmount);

    const AMOUNT = LAMPORTS_PER_SOL;

    const tx = await program.methods.addRewards(bn(AMOUNT)).accounts({}).rpc();

    console.log("add_rewards tx signature:", tx);

    const configAccount = await getConfigAccount(connection, config);
    expect(Number(configAccount?.rewardAmount)).eq(preAmount + AMOUNT);
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

    const simulationTx = await simulateTransaction(connection, tx);
    console.log("cu consumed:", simulationTx.value.unitsConsumed);

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
    const AMOUNT = LAMPORTS_PER_SOL;

    const tx = await program.methods
      .deposit(bn(AMOUNT))
      .accounts({
        farmer: wallet.publicKey,
        mint: argMint,
      })
      .rpc();

    console.log("deposit tx signature:", tx);

    // checks
    const [market] = await findMarketPda({ mint: address(argMint.toString()) });
    const marketAccount = await getMarketAccount(connection, market);

    const fee = ((marketAccount?.feeBps as number) * AMOUNT) / 10_000; // in bps
    const amountSubFee = AMOUNT - fee;

    expect(Number(marketAccount?.collectedFees)).eq(fee);
    expect(Number(marketAccount?.totalDeposited)).eq(amountSubFee);
  });

  it("`withdraw` ix", async () => {
    const [market] = await findMarketPda({ mint: address(argMint.toString()) });
    const preMarketAccount = await getMarketAccount(connection, market);

    const AMOUNT = LAMPORTS_PER_SOL / 2;

    const tx = await program.methods
      .withdraw(bn(AMOUNT))
      .accounts({
        mint: argMint,
      })
      .rpc();

    console.log("withdraw tx signature:", tx);

    // checks
    const marketAccount = await getMarketAccount(connection, market);
    expect(Number(marketAccount?.totalDeposited)).eq(
      Number(preMarketAccount?.totalDeposited) - AMOUNT,
    );
  });

  it("`settle_market` manual ix", async () => {
    const status = { winner: {} };

    const tx = await program.methods
      .settleMarket(status)
      .accounts({
        mint: argMint,
      })
      .rpc();

    console.log("settle_market tx signature:", tx);

    const [config] = await findConfigPda();
    const [market] = await findMarketPda({ mint: address(argMint.toString()) });
    const configAccount = await getConfigAccount(connection, config);
    const marketAccount = await getMarketAccount(connection, market);

    expect(configAccount?.winnerSettled).to.be.true;
    expect(marketAccount?.status).eq(MarketStatus.Winner);
  });

  it(`claim_rewards ix`, async () => {
    const [market] = await findMarketPda({ mint: address(argMint.toString()) });
    const preMarketAccount = await getMarketAccount(connection, market);

    const ix = await program.methods.claimRewards().accounts({ mint: argMint }).instruction();

    const receiptMint = new PublicKey((preMarketAccount?.receiptMint as anchor.Address).toString());
    const farmerReceiptAta = getAssociatedTokenAddressSync(
      receiptMint,
      wallet.publicKey,
      false,
      TOKEN_2022_PROGRAM_ID,
    );

    const closeAtaIx = createCloseAccountInstruction(
      farmerReceiptAta,
      wallet.publicKey,
      wallet.publicKey,
      undefined,
      TOKEN_2022_PROGRAM_ID,
    );

    const tx = new Transaction().add(ix, closeAtaIx);
    tx.feePayer = wallet.publicKey;
    tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;

    const sig = await sendAndConfirmTransaction(connection, tx, [payer]);

    console.log("claim_rewards tx signature:", sig);

    // checks
    const [config] = await findConfigPda();

    const configAccount = await getConfigAccount(connection, config);
    const marketAccount = await getMarketAccount(connection, market);

    expect(Number(configAccount?.totalClaimed)).greaterThan(0);
    expect(Number(marketAccount?.totalClaimed)).greaterThan(0);
  });

  it("`claim_fees` ix", async () => {
    const tx = await program.methods
      .claimFees()
      .accounts({
        mint: argMint,
      })
      .rpc();

    console.log("claim_fees tx signature:", tx);

    const [market] = await findMarketPda({ mint: address(argMint.toString()) });
    const marketAccount = await getMarketAccount(connection, market);

    expect(marketAccount?.feesClaimed).to.be.true;
  });
});
