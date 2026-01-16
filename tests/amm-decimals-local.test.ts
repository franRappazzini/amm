import * as anchor from "@coral-xyz/anchor";

import {
  Account,
  TOKEN_PROGRAM_ID,
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
} from "@solana/spl-token";
import { ComputeBudgetProgram, LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import {
  bn,
  calculateDepositExcess,
  initialMintLiquidity,
  subsequentMintLiquidity,
} from "./utils/functions";
import { getGlobalConfigAccount, getLiquidityPoolAccount } from "./utils/accounts";

import { Amm } from "../target/types/amm";
import { MINIMUM_LIQUIDITY } from "./utils/constants";
import { Program } from "@coral-xyz/anchor";
import { expect } from "chai";
import { getSimulationComputeUnits } from "@solana-developers/helpers";

describe("amm - decimals tests", () => {
  const provider = anchor.AnchorProvider.env();
  const { connection, wallet } = provider;

  anchor.setProvider(provider);

  const program = anchor.workspace.amm as Program<Amm>;

  const liquidityProvider = anchor.web3.Keypair.generate();
  const feeBps = 25;
  const protocolFeeBps = 5;

  before(async () => {
    await connection.requestAirdrop(liquidityProvider.publicKey, LAMPORTS_PER_SOL * 2);
    await new Promise((resolve) => setTimeout(resolve, 1000));
  });

  describe("Validation of maximum decimals", () => {
    it("Should reject mints with more than 12 decimals - mint A", async () => {
      const mintA = await createMint(connection, wallet.payer, wallet.publicKey, null, 13);
      const mintB = await createMint(connection, wallet.payer, wallet.publicKey, null, 6);

      await program.methods
        .initialize(protocolFeeBps, feeBps)
        .accounts({
          authority: wallet.publicKey,
        })
        .rpc();

      const [globalConfig] = await getGlobalConfigAccount();
      const poolCount = globalConfig.poolCount;

      const lpProvAtaA = await getOrCreateAssociatedTokenAccount(
        connection,
        liquidityProvider,
        mintA,
        liquidityProvider.publicKey,
      );

      const lpProvAtaB = await getOrCreateAssociatedTokenAccount(
        connection,
        liquidityProvider,
        mintB,
        liquidityProvider.publicKey,
      );

      await mintTo(
        connection,
        wallet.payer,
        mintA,
        lpProvAtaA.address,
        wallet.publicKey,
        1_000_000_000_000,
      );

      await mintTo(
        connection,
        wallet.payer,
        mintB,
        lpProvAtaB.address,
        wallet.publicKey,
        1_000_000_000_000,
      );

      try {
        await program.methods
          .createLiquidityPool(bn(100_000_000), bn(100_000_000))
          .accounts({
            tokenProgram: TOKEN_PROGRAM_ID,
            creator: liquidityProvider.publicKey,
            mintA: mintA,
            mintB: mintB,
          })
          .signers([liquidityProvider])
          .rpc();

        expect.fail("Should have thrown error for decimals > 12");
      } catch (error) {
        expect(error.toString()).to.include("DecimalsTooLarge");
      }
    });

    it("Should reject mints with more than 12 decimals - mint B", async () => {
      const mintA = await createMint(connection, wallet.payer, wallet.publicKey, null, 6);
      const mintB = await createMint(connection, wallet.payer, wallet.publicKey, null, 15);

      const lpProvAtaA = await getOrCreateAssociatedTokenAccount(
        connection,
        liquidityProvider,
        mintA,
        liquidityProvider.publicKey,
      );

      const lpProvAtaB = await getOrCreateAssociatedTokenAccount(
        connection,
        liquidityProvider,
        mintB,
        liquidityProvider.publicKey,
      );

      await mintTo(
        connection,
        wallet.payer,
        mintA,
        lpProvAtaA.address,
        wallet.publicKey,
        1_000_000_000_000,
      );

      await mintTo(
        connection,
        wallet.payer,
        mintB,
        lpProvAtaB.address,
        wallet.publicKey,
        1_000_000_000_000,
      );

      try {
        await program.methods
          .createLiquidityPool(bn(100_000_000), bn(100_000_000))
          .accounts({
            tokenProgram: TOKEN_PROGRAM_ID,
            creator: liquidityProvider.publicKey,
            mintA: mintA,
            mintB: mintB,
          })
          .signers([liquidityProvider])
          .rpc();

        expect.fail("Should have thrown error for decimals > 12");
      } catch (error) {
        expect(error.toString()).to.include("DecimalsTooLarge");
      }
    });

    it("Should accept mints with 12 decimals", async () => {
      const mintA = await createMint(connection, wallet.payer, wallet.publicKey, null, 12);
      const mintB = await createMint(connection, wallet.payer, wallet.publicKey, null, 12);

      const lpProvAtaA = await getOrCreateAssociatedTokenAccount(
        connection,
        liquidityProvider,
        mintA,
        liquidityProvider.publicKey,
      );

      const lpProvAtaB = await getOrCreateAssociatedTokenAccount(
        connection,
        liquidityProvider,
        mintB,
        liquidityProvider.publicKey,
      );

      await mintTo(
        connection,
        wallet.payer,
        mintA,
        lpProvAtaA.address,
        wallet.publicKey,
        1_000_000_000_000_000,
      );

      await mintTo(
        connection,
        wallet.payer,
        mintB,
        lpProvAtaB.address,
        wallet.publicKey,
        1_000_000_000_000_000,
      );

      await new Promise((resolve) => setTimeout(resolve, 1000));

      const ix = await program.methods
        .createLiquidityPool(bn(1_000_000_000), bn(1_000_000_000))
        .accounts({
          tokenProgram: TOKEN_PROGRAM_ID,
          creator: liquidityProvider.publicKey,
          mintA: mintA,
          mintB: mintB,
        })
        .instruction();

      // calculate compute units of ix
      const computeUnits = await getSimulationComputeUnits(
        connection,
        [ix],
        liquidityProvider.publicKey,
        [],
      );

      console.log("Estimated compute units:", computeUnits);

      const computeUnitIx = ComputeBudgetProgram.setComputeUnitLimit({ units: computeUnits });

      const tx = new anchor.web3.Transaction().add(computeUnitIx, ix);

      const txSignature = await anchor.web3.sendAndConfirmTransaction(connection, tx, [
        liquidityProvider,
      ]);

      expect(txSignature).to.be.a("string");
    });
  });

  describe("Pool operations with different decimals (6 and 9)", () => {
    let mintA: PublicKey;
    let mintB: PublicKey;
    let poolId: number;
    let lpProvAtaA: Account;
    let lpProvAtaB: Account;

    before(async () => {
      // Create mints with different decimals (simulating USDC=6 and SOL=9)
      mintA = await createMint(connection, wallet.payer, wallet.publicKey, null, 6);
      mintB = await createMint(connection, wallet.payer, wallet.publicKey, null, 9);

      console.log("Mint A (6 decimals):", mintA.toBase58());
      console.log("Mint B (9 decimals):", mintB.toBase58());

      lpProvAtaA = await getOrCreateAssociatedTokenAccount(
        connection,
        liquidityProvider,
        mintA,
        liquidityProvider.publicKey,
      );

      lpProvAtaB = await getOrCreateAssociatedTokenAccount(
        connection,
        liquidityProvider,
        mintB,
        liquidityProvider.publicKey,
      );

      // Mint 1,000 USDC (1,000 * 10^6) and 10 SOL (10 * 10^9)
      await mintTo(
        connection,
        wallet.payer,
        mintA,
        lpProvAtaA.address,
        wallet.publicKey,
        1_000_000_000_000, // 1M USDC for testing
      );

      await mintTo(
        connection,
        wallet.payer,
        mintB,
        lpProvAtaB.address,
        wallet.publicKey,
        100_000_000_000_000, // 100k SOL for testing
      );

      const [globalConfig] = await getGlobalConfigAccount();
      poolId = globalConfig.poolCount;
    });

    it("Should create pool with different decimals", async () => {
      const amountA = 100_000_000; // 100 USDC (6 decimals)
      const amountB = 10_000_000_000; // 0.01 SOL (9 decimals)

      await mintTo(
        connection,
        wallet.payer,
        mintA,
        lpProvAtaA.address,
        wallet.publicKey,
        1_000_000_000_000,
      );

      await mintTo(
        connection,
        wallet.payer,
        mintB,
        lpProvAtaB.address,
        wallet.publicKey,
        1_000_000_000_000_000,
      );

      await new Promise((resolve) => setTimeout(resolve, 1000));

      const ix = await program.methods
        .createLiquidityPool(bn(amountA), bn(amountB))
        .accounts({
          creator: liquidityProvider.publicKey,
          mintA: mintA,
          mintB: mintB,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .instruction();

      // calculate compute units of ix
      const computeUnits = await getSimulationComputeUnits(
        connection,
        [ix],
        liquidityProvider.publicKey,
        [],
      );

      console.log("Estimated compute units:", computeUnits);

      const computeUnitIx = ComputeBudgetProgram.setComputeUnitLimit({ units: computeUnits });

      const tx = new anchor.web3.Transaction().add(computeUnitIx, ix);

      const txSignature = await anchor.web3.sendAndConfirmTransaction(connection, tx, [
        liquidityProvider,
      ]);

      expect(txSignature).to.be.a("string");

      const [pool] = await getLiquidityPoolAccount(poolId);

      expect(pool.amountMintA).to.equal(amountA);
      expect(pool.amountMintB).to.equal(amountB);

      const expectedLiquidity = initialMintLiquidity(amountA, amountB) + MINIMUM_LIQUIDITY;
      expect(pool.lpSupply).eq(expectedLiquidity);
    });

    it("Should deposit liquidity with exact proportions", async () => {
      const [poolBefore] = await getLiquidityPoolAccount(poolId);

      const amountA = 100_000_000; // 100 USDC
      const amountB = 10_000_000_000; // 10 SOL (proportional to pool 100:10)

      const expectedLiquidity = subsequentMintLiquidity(
        amountA,
        amountB,
        poolBefore.lpSupply,
        poolBefore.amountMintA,
        poolBefore.amountMintB,
      );

      await new Promise((resolve) => setTimeout(resolve, 1000));

      const tx = await program.methods
        .depositLiquidity(bn(poolId), bn(amountA), bn(amountB), bn(0))
        .accounts({
          provider: liquidityProvider.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([liquidityProvider])
        .rpc();

      expect(tx).to.be.a("string");

      const [poolAfter] = await getLiquidityPoolAccount(poolId);

      expect(poolAfter.amountMintA).to.equal(poolBefore.amountMintA + amountA);
      expect(poolAfter.amountMintB).to.equal(poolBefore.amountMintB + amountB);
      expect(poolAfter.lpSupply).to.equal(poolBefore.lpSupply + expectedLiquidity);
    });

    it("Should deposit liquidity with excess token A", async () => {
      const [poolBefore] = await getLiquidityPoolAccount(poolId);

      const amountA = 150_000_000; // 150 USDC (excess)
      const amountB = 1_000_000_000; // 1 SOL

      const { newAmountA, newAmountB } = calculateDepositExcess(
        amountA,
        amountB,
        poolBefore.amountMintA,
        poolBefore.amountMintB,
      );

      console.log("Amounts after excess calculation:", { newAmountA, newAmountB });

      const tx = await program.methods
        .depositLiquidity(bn(poolId), bn(amountA), bn(amountB), bn(0))
        .accounts({
          tokenProgram: TOKEN_PROGRAM_ID,
          provider: liquidityProvider.publicKey,
        })
        .signers([liquidityProvider])
        .rpc();

      expect(tx).to.be.a("string");

      const [poolAfter] = await getLiquidityPoolAccount(poolId);

      expect(poolAfter.amountMintA).to.equal(poolBefore.amountMintA + newAmountA);
      expect(poolAfter.amountMintB).to.equal(poolBefore.amountMintB + newAmountB);
    });

    it("Should deposit liquidity with excess token B", async () => {
      const [poolBefore] = await getLiquidityPoolAccount(poolId);

      const amountA = 50_000_000; // 50 USDC
      const amountB = 1_000_000_000; // 1 SOL (excess)

      const { newAmountA, newAmountB } = calculateDepositExcess(
        amountA,
        amountB,
        poolBefore.amountMintA,
        poolBefore.amountMintB,
      );

      console.log("Amounts after excess calculation:", { newAmountA, newAmountB });

      const tx = await program.methods
        .depositLiquidity(bn(poolId), bn(amountA), bn(amountB), bn(0))
        .accounts({
          tokenProgram: TOKEN_PROGRAM_ID,
          provider: liquidityProvider.publicKey,
        })
        .signers([liquidityProvider])
        .rpc();

      expect(tx).to.be.a("string");

      const [poolAfter] = await getLiquidityPoolAccount(poolId);

      expect(poolAfter.amountMintA).to.equal(poolBefore.amountMintA + newAmountA);
      expect(poolAfter.amountMintB).to.equal(poolBefore.amountMintB + newAmountB);
    });

    it("Should swap with different decimals (A to B)", async () => {
      const [poolBefore] = await getLiquidityPoolAccount(poolId);

      const inputAmount = 10_000_000; // 10 USDC

      const tx = await program.methods
        .swap(bn(poolId), { exactIn: { inputAmount: bn(inputAmount) } }, bn(0))
        .accounts({
          tokenProgram: TOKEN_PROGRAM_ID,
          signer: liquidityProvider.publicKey,
          inputMint: mintA,
          outputMint: mintB,
        })
        .signers([liquidityProvider])
        .rpc();

      expect(tx).to.be.a("string");

      const [poolAfter] = await getLiquidityPoolAccount(poolId);

      expect(poolAfter.amountMintA).to.be.greaterThan(poolBefore.amountMintA);
      expect(poolAfter.amountMintB).to.be.lessThan(poolBefore.amountMintB);
    });

    it("Should swap with different decimals (B to A)", async () => {
      const [poolBefore] = await getLiquidityPoolAccount(poolId);

      const inputAmount = 100_000_000; // 0.1 SOL

      const tx = await program.methods
        .swap(bn(poolId), { exactIn: { inputAmount: bn(inputAmount) } }, bn(0))
        .accounts({
          tokenProgram: TOKEN_PROGRAM_ID,
          signer: liquidityProvider.publicKey,
          inputMint: mintB,
          outputMint: mintA,
        })
        .signers([liquidityProvider])
        .rpc();

      expect(tx).to.be.a("string");

      const [poolAfter] = await getLiquidityPoolAccount(poolId);

      expect(poolAfter.amountMintB).to.be.greaterThan(poolBefore.amountMintB);
      expect(poolAfter.amountMintA).to.be.lessThan(poolBefore.amountMintA);
    });

    it("Should redeem LP tokens correctly with different decimals", async () => {
      const [lpMintPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("lp_mint"), bn(poolId).toArrayLike(Buffer, "le", 8)],
        program.programId,
      );

      const lpAta = await getOrCreateAssociatedTokenAccount(
        connection,
        liquidityProvider,
        lpMintPda,
        liquidityProvider.publicKey,
      );

      const lpBalance = lpAta.amount;
      const redeemAmount = Math.floor(Number(lpBalance) / 10); // Redeem 10%

      const [poolBefore] = await getLiquidityPoolAccount(poolId);

      const tx = await program.methods
        .redeemLp(bn(poolId), bn(redeemAmount))
        .accounts({
          tokenProgram: TOKEN_PROGRAM_ID,
          redeemer: liquidityProvider.publicKey,
        })
        .signers([liquidityProvider])
        .rpc();

      expect(tx).to.be.a("string");

      const [poolAfter] = await getLiquidityPoolAccount(poolId);

      expect(poolAfter.lpSupply).to.equal(poolBefore.lpSupply - redeemAmount);
      expect(poolAfter.amountMintA).to.be.lessThan(poolBefore.amountMintA);
      expect(poolAfter.amountMintB).to.be.lessThan(poolBefore.amountMintB);
    });
  });

  describe("Pool operations with edge case decimals (0 and 12)", () => {
    let mintA: PublicKey;
    let mintB: PublicKey;
    let poolId: number;

    before(async () => {
      mintA = await createMint(connection, wallet.payer, wallet.publicKey, null, 0);
      mintB = await createMint(connection, wallet.payer, wallet.publicKey, null, 12);

      console.log("Mint A (0 decimals):", mintA.toBase58());
      console.log("Mint B (12 decimals):", mintB.toBase58());

      const lpProvAtaA = await getOrCreateAssociatedTokenAccount(
        connection,
        liquidityProvider,
        mintA,
        liquidityProvider.publicKey,
      );

      const lpProvAtaB = await getOrCreateAssociatedTokenAccount(
        connection,
        liquidityProvider,
        mintB,
        liquidityProvider.publicKey,
      );

      await mintTo(
        connection,
        wallet.payer,
        mintA,
        lpProvAtaA.address,
        wallet.publicKey,
        1_000_000,
      );

      await mintTo(
        connection,
        wallet.payer,
        mintB,
        lpProvAtaB.address,
        wallet.publicKey,
        1_000_000_000_000_000,
      );

      const [globalConfig] = await getGlobalConfigAccount();
      poolId = globalConfig.poolCount;

      await new Promise((resolve) => setTimeout(resolve, 1000));
    });

    it("Should create pool with extreme decimal differences (0 and 12)", async () => {
      const amountA = 10_000; // 10,000 tokens (0 decimals)
      const amountB = 10_000_000_000; // 0.010 tokens (12 decimals)

      const ix = await program.methods
        .createLiquidityPool(bn(amountA), bn(amountB))
        .accounts({
          tokenProgram: TOKEN_PROGRAM_ID,
          creator: liquidityProvider.publicKey,
          mintA: mintA,
          mintB: mintB,
        })
        .instruction();

      // calculate compute units of ix
      const computeUnits = await getSimulationComputeUnits(
        connection,
        [ix],
        liquidityProvider.publicKey,
        [],
      );

      console.log("Estimated compute units:", computeUnits);

      const computeUnitIx = ComputeBudgetProgram.setComputeUnitLimit({ units: computeUnits });

      const tx = new anchor.web3.Transaction().add(computeUnitIx, ix);

      const txSignature = await anchor.web3.sendAndConfirmTransaction(connection, tx, [
        liquidityProvider,
      ]);

      expect(txSignature).to.be.a("string");

      const [pool] = await getLiquidityPoolAccount(poolId);

      expect(pool.amountMintA).to.equal(amountA);
      expect(pool.amountMintB).to.equal(amountB);
    });

    it("Should deposit and calculate excess correctly with extreme decimals", async () => {
      const [poolBefore] = await getLiquidityPoolAccount(poolId);

      const amountA = 100; // 100 tokens (0 decimals)
      const amountB = 200_000_000_000_000; // 200 tokens (12 decimals) - excess

      const { newAmountA, newAmountB } = calculateDepositExcess(
        amountA,
        amountB,
        poolBefore.amountMintA,
        poolBefore.amountMintB,
      );

      const tx = await program.methods
        .depositLiquidity(bn(poolId), bn(amountA), bn(amountB), bn(0))
        .accounts({
          tokenProgram: TOKEN_PROGRAM_ID,
          provider: liquidityProvider.publicKey,
        })
        .signers([liquidityProvider])
        .rpc();

      expect(tx).to.be.a("string");

      const [poolAfter] = await getLiquidityPoolAccount(poolId);

      expect(poolAfter.amountMintA).to.equal(poolBefore.amountMintA + newAmountA);
      expect(poolAfter.amountMintB).to.equal(poolBefore.amountMintB + newAmountB);
    });
  });
});
