import * as anchor from "@coral-xyz/anchor";

import { ComputeBudgetProgram, LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
} from "@solana/spl-token";
import { bn, initialMintLiquidity, subsequentMintLiquidity } from "./utils/functions";
import { getGlobalConfigAccount, getLiquidityPoolAccount } from "./utils/accounts";

import { Amm } from "../target/types/amm";
import { MINIMUM_LIQUIDITY } from "./utils/constants";
import { Program } from "@coral-xyz/anchor";
import { expect } from "chai";
import { getSimulationComputeUnits } from "@solana-developers/helpers";

describe("amm", () => {
  const provider = anchor.AnchorProvider.env();
  const { connection, wallet } = provider;

  anchor.setProvider(provider);

  const program = anchor.workspace.amm as Program<Amm>;

  const liquidityProvider = anchor.web3.Keypair.generate();

  let mintA: PublicKey;
  let mintB: PublicKey;

  before(async () => {
    await connection.requestAirdrop(liquidityProvider.publicKey, LAMPORTS_PER_SOL);

    // tokens mint creation
    mintA = await createMint(connection, wallet.payer, wallet.publicKey, null, 6);
    mintB = await createMint(connection, wallet.payer, wallet.publicKey, null, 6);

    console.log("Mint A:", mintA.toBase58());
    console.log("Mint B:", mintB.toBase58());

    // atas creation
    const liqProvAtaA = await getOrCreateAssociatedTokenAccount(
      connection,
      liquidityProvider,
      mintA,
      liquidityProvider.publicKey
    );

    const liqProvAtaB = await getOrCreateAssociatedTokenAccount(
      connection,
      liquidityProvider,
      mintB,
      liquidityProvider.publicKey
    );

    // mint tokens to atas
    await mintTo(
      connection,
      wallet.payer,
      mintA,
      liqProvAtaA.address,
      wallet.publicKey,
      1_000_000_000
    ); // 1,000 tokens

    await mintTo(
      connection,
      wallet.payer,
      mintB,
      liqProvAtaB.address,
      wallet.publicKey,
      2_000_000_000
    ); // 2,000 tokens
  });

  it("`initialize` method!", async () => {
    const FEE_BPS = 30; // 0.3%

    const tx = await program.methods.initialize(FEE_BPS).rpc();
    console.log("`initialize` tx signature:", tx);

    const [globalConfig] = await getGlobalConfigAccount();

    expect(globalConfig.authority).eq(wallet.publicKey.toBase58());
    expect(globalConfig.feeBps).eq(FEE_BPS);
    
    await new Promise((resolve) => setTimeout(resolve, 1000)); // wait for 1 seconds to avoid tx failure
  });

  it("`create_liquidity_pool` method!", async () => {

    const AMOUNT_A = 10_000_000; // 10 tokens
    const AMOUNT_B = 20_000_000; // 20 tokens

    const ix = await program.methods
      .createLiquidityPool(bn(AMOUNT_A), bn(AMOUNT_B))
      .accounts({
        creator: liquidityProvider.publicKey,
        mintA,
        mintB,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .instruction();

    // calculate compute units of ix
    const computeUnits = await getSimulationComputeUnits(
      connection,
      [ix],
      liquidityProvider.publicKey,
      []
    );

    console.log("Estimated compute units:", computeUnits);

    const computeUnitIx = ComputeBudgetProgram.setComputeUnitLimit({ units: computeUnits });

    const tx = new anchor.web3.Transaction().add(computeUnitIx, ix);

    const txSignature = await anchor.web3.sendAndConfirmTransaction(connection, tx, [
      liquidityProvider,
    ]);

    console.log("`create_liquidity_pool` tx signature:", txSignature);

    const [globalConfig] = await getGlobalConfigAccount();
    const [liquidityPool] = await getLiquidityPoolAccount(0);

    const liquidity = initialMintLiquidity(AMOUNT_A, AMOUNT_B) + MINIMUM_LIQUIDITY;

    expect(globalConfig.poolCount).eq(1);
    expect(liquidityPool.creator).eq(liquidityProvider.publicKey.toBase58());
    expect(liquidityPool.amountMintA).eq(AMOUNT_A);
    expect(liquidityPool.amountMintB).eq(AMOUNT_B);
    expect(liquidityPool.lpSupply).eq(liquidity);
  });

  it("`deposit_liquidity` method!", async () => {
    const poolId = 0;
    const AMOUNT_A = 5_000_000; // 5 tokens
    const AMOUNT_B = 10_000_000; // 10 tokens

    const [prevLiquidityPool] = await getLiquidityPoolAccount(poolId);

    const tx = await program.methods
      .depositLiquidity(bn(poolId), bn(AMOUNT_A), bn(AMOUNT_B))
      .accounts({
        provider: liquidityProvider.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([liquidityProvider])
      .rpc();

    console.log("`deposit_liquidity` tx signature:", tx);

    const liquidity = subsequentMintLiquidity(
      AMOUNT_A,
      AMOUNT_B,
      prevLiquidityPool.lpSupply,
      prevLiquidityPool.amountMintA,
      prevLiquidityPool.amountMintB
    );

    const [liquidityPool] = await getLiquidityPoolAccount(poolId);

    expect(liquidityPool.amountMintA).eq(prevLiquidityPool.amountMintA + AMOUNT_A);
    expect(liquidityPool.amountMintB).eq(prevLiquidityPool.amountMintB + AMOUNT_B);
    expect(liquidityPool.lpSupply).eq(prevLiquidityPool.lpSupply + liquidity);
  });
});
