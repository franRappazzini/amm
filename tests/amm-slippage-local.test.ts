import * as anchor from "@coral-xyz/anchor";

import { ComputeBudgetProgram, LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
} from "@solana/spl-token";
import { bn, calculateSwapAmounts, subsequentMintLiquidity } from "./utils/functions";
import { getGlobalConfigAccount, getLiquidityPoolAccount } from "./utils/accounts";

import { Amm } from "../target/types/amm";
import { MINIMUM_LIQUIDITY } from "./utils/constants";
import { Program } from "@coral-xyz/anchor";
import { expect } from "chai";
import { getSimulationComputeUnits } from "@solana-developers/helpers";

describe("amm - slippage protection tests", () => {
  const provider = anchor.AnchorProvider.env();
  const { connection, wallet } = provider;

  anchor.setProvider(provider);

  const program = anchor.workspace.amm as Program<Amm>;

  const liquidityProvider = anchor.web3.Keypair.generate();
  const trader = anchor.web3.Keypair.generate();
  const feeBps = 25;
  const protocolFeeBps = 5;

  let mintA: PublicKey;
  let mintB: PublicKey;
  let poolId: number;

  before(async () => {
    await connection.requestAirdrop(liquidityProvider.publicKey, LAMPORTS_PER_SOL * 2);
    await connection.requestAirdrop(trader.publicKey, LAMPORTS_PER_SOL * 2);
    await new Promise((resolve) => setTimeout(resolve, 1000));

    // Create mints
    mintA = await createMint(connection, wallet.payer, wallet.publicKey, null, 6);
    mintB = await createMint(connection, wallet.payer, wallet.publicKey, null, 6);

    console.log("Mint A:", mintA.toBase58());
    console.log("Mint B:", mintB.toBase58());

    // Create ATAs and mint tokens
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

    const traderAtaA = await getOrCreateAssociatedTokenAccount(
      connection,
      trader,
      mintA,
      trader.publicKey,
    );

    await mintTo(
      connection,
      wallet.payer,
      mintA,
      lpProvAtaA.address,
      wallet.publicKey,
      10_000_000_000_000, // 10M tokens
    );

    await mintTo(
      connection,
      wallet.payer,
      mintB,
      lpProvAtaB.address,
      wallet.publicKey,
      10_000_000_000_000, // 10M tokens
    );

    await mintTo(
      connection,
      wallet.payer,
      mintA,
      traderAtaA.address,
      wallet.publicKey,
      1_000_000_000_000, // 1M tokens
    );

    await program.methods
      .initialize(protocolFeeBps, feeBps)
      .accounts({
        authority: wallet.publicKey,
      })
      .rpc();

    await new Promise((resolve) => setTimeout(resolve, 1000));

    const [globalConfig] = await getGlobalConfigAccount();
    poolId = globalConfig.poolCount;

    // Create pool

    const ix = await program.methods
      .createLiquidityPool(bn(1_000_000_000), bn(1_000_000_000))
      .accounts({
        creator: liquidityProvider.publicKey,
        mintA: mintA,
        mintB: mintB,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([liquidityProvider])
      .instruction();

    // calculate compute units of ix
    const computeUnits = await getSimulationComputeUnits(
      connection,
      [ix],
      liquidityProvider.publicKey,
      [],
    );

    const computeUnitIx = ComputeBudgetProgram.setComputeUnitLimit({ units: computeUnits });

    const tx = new anchor.web3.Transaction().add(computeUnitIx, ix);

    await anchor.web3.sendAndConfirmTransaction(connection, tx, [liquidityProvider]);

    console.log("Pool created with ID:", poolId);
  });

  describe("Deposit Liquidity Slippage Protection", () => {
    it("Should succeed when min_lp_out is met", async () => {
      const [poolBefore] = await getLiquidityPoolAccount(poolId);

      const amountA = 100_000_000; // 100 tokens
      const amountB = 100_000_000; // 100 tokens

      const expectedLiquidity = subsequentMintLiquidity(
        amountA,
        amountB,
        poolBefore.lpSupply,
        poolBefore.amountMintA,
        poolBefore.amountMintB,
      );

      // Set min_lp_out to 90% of expected
      const minLpOut = Math.floor(expectedLiquidity * 0.9);

      const tx = await program.methods
        .depositLiquidity(bn(poolId), bn(amountA), bn(amountB), bn(minLpOut))
        .accounts({
          provider: liquidityProvider.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([liquidityProvider])
        .rpc();

      expect(tx).to.be.a("string");

      const [poolAfter] = await getLiquidityPoolAccount(poolId);

      expect(poolAfter.lpSupply).to.be.greaterThanOrEqual(poolBefore.lpSupply + minLpOut);
    });

    it("Should fail when min_lp_out is too high", async () => {
      const [poolBefore] = await getLiquidityPoolAccount(poolId);

      const amountA = 100_000_000; // 100 tokens
      const amountB = 100_000_000; // 100 tokens

      const expectedLiquidity = subsequentMintLiquidity(
        amountA,
        amountB,
        poolBefore.lpSupply,
        poolBefore.amountMintA,
        poolBefore.amountMintB,
      );

      // Set min_lp_out to 200% of expected (unrealistic)
      const minLpOut = expectedLiquidity * 2;

      try {
        await program.methods
          .depositLiquidity(bn(poolId), bn(amountA), bn(amountB), bn(minLpOut))
          .accounts({
            provider: liquidityProvider.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([liquidityProvider])
          .rpc();

        expect.fail("Should have thrown SlippageExceeded error");
      } catch (error) {
        expect(error.toString()).to.include("SlippageExceeded");
      }
    });

    it("Should protect against front-running in deposit", async () => {
      const [poolBefore] = await getLiquidityPoolAccount(poolId);

      const amountA = 50_000_000; // 50 tokens
      const amountB = 50_000_000; // 50 tokens

      const expectedLiquidity = subsequentMintLiquidity(
        amountA,
        amountB,
        poolBefore.lpSupply,
        poolBefore.amountMintA,
        poolBefore.amountMintB,
      );

      // User expects exact amount
      const minLpOut = expectedLiquidity;

      // Simulate front-running: another deposit happens first
      await program.methods
        .depositLiquidity(bn(poolId), bn(10_000_000), bn(10_000_000), bn(0))
        .accounts({
          provider: liquidityProvider.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([liquidityProvider])
        .rpc();

      // Original transaction should still succeed if slippage is reasonable
      const tx = await program.methods
        .depositLiquidity(bn(poolId), bn(amountA), bn(amountB), bn(Math.floor(minLpOut * 0.95)))
        .accounts({
          provider: liquidityProvider.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([liquidityProvider])
        .rpc();

      expect(tx).to.be.a("string");
    });
  });

  describe("Swap ExactIn Slippage Protection", () => {
    it("Should succeed when min_amount_out is met", async () => {
      const [pool] = await getLiquidityPoolAccount(poolId);

      const inputAmount = 10_000_000; // 10 tokens

      const { outputAmount } = calculateSwapAmounts(
        { exactIn: { inputAmount: bn(inputAmount) } },
        pool.amountMintA,
        pool.amountMintB,
        feeBps,
        protocolFeeBps,
      );

      // Set slippage_limit to 95% of expected output
      const minAmountOut = Math.floor(outputAmount * 0.95);

      const tx = await program.methods
        .swap(bn(poolId), { exactIn: { inputAmount: bn(inputAmount) } }, bn(minAmountOut))
        .accounts({
          signer: trader.publicKey,
          inputMint: mintA,
          outputMint: mintB,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([trader])
        .rpc();

      expect(tx).to.be.a("string");
    });

    it("Should fail when min_amount_out is too high", async () => {
      const [pool] = await getLiquidityPoolAccount(poolId);

      const inputAmount = 10_000_000; // 10 tokens

      const { outputAmount } = calculateSwapAmounts(
        { exactIn: { inputAmount: bn(inputAmount) } },
        pool.amountMintA,
        pool.amountMintB,
        feeBps,
        protocolFeeBps,
      );

      // Set slippage_limit to 150% of expected output (unrealistic)
      const minAmountOut = Math.floor(outputAmount * 1.5);

      try {
        await program.methods
          .swap(bn(poolId), { exactIn: { inputAmount: bn(inputAmount) } }, bn(minAmountOut))
          .accounts({
            signer: trader.publicKey,
            inputMint: mintA,
            outputMint: mintB,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([trader])
          .rpc();

        expect.fail("Should have thrown SlippageExceeded error");
      } catch (error) {
        expect(error.toString()).to.include("SlippageExceeded");
      }
    });

    it("Should protect against sandwich attack in ExactIn swap", async () => {
      const [poolBefore] = await getLiquidityPoolAccount(poolId);

      const victimInputAmount = 50_000_000; // 50 tokens

      const { outputAmount: expectedOutput } = calculateSwapAmounts(
        { exactIn: { inputAmount: bn(victimInputAmount) } },
        poolBefore.amountMintA,
        poolBefore.amountMintB,
        feeBps,
        protocolFeeBps,
      );

      // Attacker front-runs with large swap
      await program.methods
        .swap(bn(poolId), { exactIn: { inputAmount: bn(100_000_000) } }, bn(0))
        .accounts({
          signer: liquidityProvider.publicKey,
          inputMint: mintA,
          outputMint: mintB,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([liquidityProvider])
        .rpc();

      // Victim's transaction with slippage protection should fail
      const minAmountOut = Math.floor(expectedOutput * 0.99); // 1% slippage tolerance

      try {
        await program.methods
          .swap(bn(poolId), { exactIn: { inputAmount: bn(victimInputAmount) } }, bn(minAmountOut))
          .accounts({
            signer: trader.publicKey,
            inputMint: mintA,
            outputMint: mintB,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([trader])
          .rpc();

        expect.fail("Should have thrown SlippageExceeded error due to sandwich attack");
      } catch (error) {
        expect(error.toString()).to.include("SlippageExceeded");
      }
    });
  });

  describe("Swap ExactOut Slippage Protection", () => {
    it("Should succeed when max_amount_in is met", async () => {
      const [pool] = await getLiquidityPoolAccount(poolId);

      const outputAmount = 5_000_000; // 5 tokens

      const { inputAmount } = calculateSwapAmounts(
        { exactOut: { outputAmount: bn(outputAmount) } },
        pool.amountMintA,
        pool.amountMintB,
        feeBps,
        protocolFeeBps,
      );

      // Set slippage_limit to 105% of expected input
      const maxAmountIn = Math.floor(inputAmount * 1.05);

      const tx = await program.methods
        .swap(bn(poolId), { exactOut: { outputAmount: bn(outputAmount) } }, bn(maxAmountIn))
        .accounts({
          signer: trader.publicKey,
          inputMint: mintA,
          outputMint: mintB,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([trader])
        .rpc();

      expect(tx).to.be.a("string");
    });

    it("Should fail when max_amount_in is too low", async () => {
      const [pool] = await getLiquidityPoolAccount(poolId);

      const outputAmount = 5_000_000; // 5 tokens

      const { inputAmount } = calculateSwapAmounts(
        { exactOut: { outputAmount: bn(outputAmount) } },
        pool.amountMintA,
        pool.amountMintB,
        feeBps,
        protocolFeeBps,
      );

      // Set slippage_limit to 50% of expected input (unrealistic)
      const maxAmountIn = Math.floor(inputAmount * 0.5);

      try {
        await program.methods
          .swap(bn(poolId), { exactOut: { outputAmount: bn(outputAmount) } }, bn(maxAmountIn))
          .accounts({
            signer: trader.publicKey,
            inputMint: mintA,
            outputMint: mintB,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([trader])
          .rpc();

        expect.fail("Should have thrown SlippageExceeded error");
      } catch (error) {
        expect(error.toString()).to.include("SlippageExceeded");
      }
    });

    it("Should protect against front-running in ExactOut swap", async () => {
      const [poolBefore] = await getLiquidityPoolAccount(poolId);

      const outputAmount = 10_000_000; // 10 tokens

      const { inputAmount: expectedInput } = calculateSwapAmounts(
        { exactOut: { outputAmount: bn(outputAmount) } },
        poolBefore.amountMintA,
        poolBefore.amountMintB,
        feeBps,
        protocolFeeBps,
      );

      // Attacker front-runs
      await program.methods
        .swap(bn(poolId), { exactIn: { inputAmount: bn(50_000_000) } }, bn(0))
        .accounts({
          signer: liquidityProvider.publicKey,
          inputMint: mintA,
          outputMint: mintB,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([liquidityProvider])
        .rpc();

      // Victim's transaction with slippage protection should fail
      const maxAmountIn = Math.floor(expectedInput * 1.01); // 1% slippage tolerance

      try {
        await program.methods
          .swap(bn(poolId), { exactOut: { outputAmount: bn(outputAmount) } }, bn(maxAmountIn))
          .accounts({
            signer: trader.publicKey,
            inputMint: mintA,
            outputMint: mintB,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([trader])
          .rpc();

        expect.fail("Should have thrown SlippageExceeded error due to front-running");
      } catch (error) {
        expect(error.toString()).to.include("SlippageExceeded");
      }
    });
  });

  describe("Edge Cases", () => {
    it("Should allow min_lp_out of 0 (no slippage protection)", async () => {
      const tx = await program.methods
        .depositLiquidity(bn(poolId), bn(10_000_000), bn(10_000_000), bn(0))
        .accounts({
          provider: liquidityProvider.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([liquidityProvider])
        .rpc();

      expect(tx).to.be.a("string");
    });

    it("Should allow slippage_limit of 0 in ExactIn (no protection)", async () => {
      const tx = await program.methods
        .swap(bn(poolId), { exactIn: { inputAmount: bn(1_000_000) } }, bn(0))
        .accounts({
          signer: trader.publicKey,
          inputMint: mintA,
          outputMint: mintB,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([trader])
        .rpc();

      expect(tx).to.be.a("string");
    });

    it("Should allow very high slippage_limit in ExactOut (no effective protection)", async () => {
      const tx = await program.methods
        .swap(
          bn(poolId),
          { exactOut: { outputAmount: bn(1_000_000) } },
          bn(1_000_000_000_000), // Very high limit
        )
        .accounts({
          signer: trader.publicKey,
          inputMint: mintA,
          outputMint: mintB,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([trader])
        .rpc();

      expect(tx).to.be.a("string");
    });
  });
});
