import * as anchor from "@coral-xyz/anchor";

import { ComputeBudgetProgram, LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
} from "@solana/spl-token";
import {
  bn,
  calculateClaimableAmount,
  calculateDepositExcess,
  calculateSwapAmounts,
  initialMintLiquidity,
  subsequentMintLiquidity,
} from "./utils/functions";
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
  let lpBalance = 0;
  const user = anchor.web3.Keypair.generate();

  let mintA: PublicKey;
  let mintB: PublicKey;

  before(async () => {
    await connection.requestAirdrop(liquidityProvider.publicKey, LAMPORTS_PER_SOL);
    await connection.requestAirdrop(user.publicKey, LAMPORTS_PER_SOL);

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

    const userAtaA = await getOrCreateAssociatedTokenAccount(
      connection,
      user,
      mintA,
      user.publicKey
    );

    // const userAtaB = await getOrCreateAssociatedTokenAccount(
    //   connection,
    //   user,
    //   mintB,
    //   user.publicKey
    // );

    // mint tokens to atas
    await mintTo(
      connection,
      wallet.payer,
      mintA,
      liqProvAtaA.address,
      wallet.publicKey,
      1_000_000_000_000
    ); // 1,000,000 tokens

    await mintTo(
      connection,
      wallet.payer,
      mintB,
      liqProvAtaB.address,
      wallet.publicKey,
      2_000_000_000_000
    ); // 2,000,000 tokens

    await mintTo(
      connection,
      wallet.payer,
      mintA,
      userAtaA.address,
      wallet.publicKey,
      1_000_000_000
    ); // 1,000 tokens

    // await mintTo(
    //   connection,
    //   wallet.payer,
    //   mintB,
    //   userAtaB.address,
    //   wallet.publicKey,
    //   2_000_000_000
    // ); // 2,000 tokens
  });

  it("`initialize` method!", async () => {
    const PROTOCOL_FEE_BPS = 5; // 0.05%
    const FEE_BPS = 25; // 0.25%

    const tx = await program.methods.initialize(PROTOCOL_FEE_BPS, FEE_BPS).rpc();
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

    lpBalance += liquidity;
    console.log("lp balance:", lpBalance);
  });

  it("`deposit_liquidity` method!", async () => {
    const poolId = 0;
    const AMOUNT_A = 500_000_000; // 500 tokens
    const AMOUNT_B = 1_000_000_000; // 1000 tokens

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

    expect(liquidityPool.amountMintA, "amount mint A").eq(prevLiquidityPool.amountMintA + AMOUNT_A);
    expect(liquidityPool.amountMintB, "amount mint B").eq(prevLiquidityPool.amountMintB + AMOUNT_B);
    expect(liquidityPool.lpSupply).eq(prevLiquidityPool.lpSupply + liquidity);

    lpBalance += liquidity;
    console.log("lp balance:", lpBalance);
  });

  it("`deposit_liquidity` method with mint A excess!", async () => {
    const poolId = 0;
    const AMOUNT_A = 1_000_000_000; // 1000 tokens
    const AMOUNT_B = 500_000_000; // 500 tokens

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
    const { newAmountA, newAmountB } = calculateDepositExcess(
      AMOUNT_A,
      AMOUNT_B,
      liquidityPool.amountMintA,
      liquidityPool.amountMintB
    );

    expect(liquidityPool.amountMintA, "amount mint A").eq(
      prevLiquidityPool.amountMintA + newAmountA
    );
    expect(liquidityPool.amountMintB, "amount mint B").eq(
      prevLiquidityPool.amountMintB + newAmountB
    );
    expect(liquidityPool.lpSupply).eq(prevLiquidityPool.lpSupply + liquidity);

    lpBalance += liquidity;
    console.log("lp balance:", lpBalance);
  });

  it("`deposit_liquidity` method with mint B excess!", async () => {
    const poolId = 0;
    const AMOUNT_A = 250_000_000; // 250 tokens
    const AMOUNT_B = 750_000_000; // 750 tokens

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
    const { newAmountA, newAmountB } = calculateDepositExcess(
      AMOUNT_A,
      AMOUNT_B,
      liquidityPool.amountMintA,
      liquidityPool.amountMintB
    );

    expect(liquidityPool.amountMintA).eq(prevLiquidityPool.amountMintA + newAmountA);
    expect(liquidityPool.amountMintB).eq(prevLiquidityPool.amountMintB + newAmountB);
    expect(liquidityPool.lpSupply).eq(prevLiquidityPool.lpSupply + liquidity);

    lpBalance += liquidity;
    console.log("lp balance:", lpBalance);
  });

  it("`redeem_lp` method!", async () => {
    const poolId = 0;
    const lpAmount = Math.floor(lpBalance / 2);

    const [prevLiquidityPool] = await getLiquidityPoolAccount(poolId);

    const tx = await program.methods
      .redeemLp(bn(poolId), bn(lpAmount))
      .accounts({
        redeemer: liquidityProvider.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([liquidityProvider])
      .rpc();

    console.log("`redeem_lp` tx signature:", tx);

    const [liquidityPool] = await getLiquidityPoolAccount(poolId);

    const claimableA = calculateClaimableAmount(
      lpAmount,
      prevLiquidityPool.lpSupply,
      prevLiquidityPool.amountMintA
    );

    const claimableB = calculateClaimableAmount(
      lpAmount,
      prevLiquidityPool.lpSupply,
      prevLiquidityPool.amountMintB
    );

    expect(liquidityPool.lpSupply).eq(prevLiquidityPool.lpSupply - lpAmount);
    expect(liquidityPool.amountMintA).eq(prevLiquidityPool.amountMintA - claimableA);
    expect(liquidityPool.amountMintB).eq(prevLiquidityPool.amountMintB - claimableB);

    lpBalance -= lpAmount;
    console.log("lp balance:", lpBalance);
  });

  it("`swap` method using 'exact in' param with mint A!", async () => {
    const poolId = 0;
    const INPUT_AMOUNT = 10_000_000; // 10 token

    const param: anchor.IdlTypes<Amm>["swapParams"] = {
      exactIn: { inputAmount: bn(INPUT_AMOUNT) },
    };

    const [prevLiquidityPool] = await getLiquidityPoolAccount(poolId);
    console.log("A amount in vault:", prevLiquidityPool.amountMintA);
    console.log("B amount in vault:", prevLiquidityPool.amountMintB);

    const tx = await program.methods
      .swap(bn(poolId), param)
      .accounts({
        signer: user.publicKey,
        inputMint: mintA,
        outputMint: mintB,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([user])
      .rpc();

    console.log("`swap` tx signature:", tx);

    const { inputAmount, outputAmount, protocolFeeAmount } = calculateSwapAmounts(
      param,
      prevLiquidityPool.amountMintA,
      prevLiquidityPool.amountMintB,
      prevLiquidityPool.feeBps,
      prevLiquidityPool.protocolFeeBps
    );

    console.log({ inputAmount, outputAmount, protocolFeeAmount });

    const [liquidityPool] = await getLiquidityPoolAccount(poolId);

    expect(liquidityPool.amountMintA, "amount mint A").eq(
      prevLiquidityPool.amountMintA + inputAmount - protocolFeeAmount
    );
    expect(liquidityPool.amountMintB, "amount mint B").eq(
      prevLiquidityPool.amountMintB - outputAmount
    );
    expect(liquidityPool.accumulatedProtocolFeeA, "accumulated fee A").eq(
      prevLiquidityPool.accumulatedProtocolFeeA + protocolFeeAmount
    );
  });

  it("`swap` method using 'exact out' param with mint A!", async () => {
    const poolId = 0;
    const OUTPUT_AMOUNT = 20_000_000; // 2 token

    const param: anchor.IdlTypes<Amm>["swapParams"] = {
      exactOut: { outputAmount: bn(OUTPUT_AMOUNT) },
    };

    const [prevLiquidityPool] = await getLiquidityPoolAccount(poolId);
    console.log("A amount in vault:", prevLiquidityPool.amountMintA);
    console.log("B amount in vault:", prevLiquidityPool.amountMintB);

    const tx = await program.methods
      .swap(bn(poolId), param)
      .accounts({
        signer: user.publicKey,
        inputMint: mintA,
        outputMint: mintB,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([user])
      .rpc();

    console.log("`swap` tx signature:", tx);

    const { inputAmount, outputAmount, protocolFeeAmount } = calculateSwapAmounts(
      param,
      prevLiquidityPool.amountMintA,
      prevLiquidityPool.amountMintB,
      prevLiquidityPool.feeBps,
      prevLiquidityPool.protocolFeeBps
    );

    console.log({ inputAmount, outputAmount, protocolFeeAmount });

    const [liquidityPool] = await getLiquidityPoolAccount(poolId);

    expect(liquidityPool.amountMintA, "amount mint A").eq(
      prevLiquidityPool.amountMintA + inputAmount - protocolFeeAmount
    );
    expect(liquidityPool.amountMintB, "amount mint B").eq(
      prevLiquidityPool.amountMintB - outputAmount
    );
    expect(liquidityPool.accumulatedProtocolFeeA, "accumulated fee A").eq(
      prevLiquidityPool.accumulatedProtocolFeeA + protocolFeeAmount
    );
  });

  it("`swap` method using 'exact in' param with mint B!", async () => {
    const poolId = 0;
    const INPUT_AMOUNT = 10_000_000; // 10 token

    const param: anchor.IdlTypes<Amm>["swapParams"] = {
      exactIn: { inputAmount: bn(INPUT_AMOUNT) },
    };

    const [prevLiquidityPool] = await getLiquidityPoolAccount(poolId);
    console.log("A amount in vault:", prevLiquidityPool.amountMintA);
    console.log("B amount in vault:", prevLiquidityPool.amountMintB);

    const tx = await program.methods
      .swap(bn(poolId), param)
      .accounts({
        signer: user.publicKey,
        inputMint: mintB,
        outputMint: mintA,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([user])
      .rpc();

    console.log("`swap` tx signature:", tx);

    const { inputAmount, outputAmount, protocolFeeAmount } = calculateSwapAmounts(
      param,
      prevLiquidityPool.amountMintB,
      prevLiquidityPool.amountMintA,
      prevLiquidityPool.feeBps,
      prevLiquidityPool.protocolFeeBps
    );

    console.log({ inputAmount, outputAmount, protocolFeeAmount });

    const [liquidityPool] = await getLiquidityPoolAccount(poolId);

    expect(liquidityPool.amountMintB, "amount mint B").eq(
      prevLiquidityPool.amountMintB + inputAmount - protocolFeeAmount
    );
    expect(liquidityPool.amountMintA, "amount mint A").eq(
      prevLiquidityPool.amountMintA - outputAmount
    );
    expect(liquidityPool.accumulatedProtocolFeeB, "accumulated fee B").eq(
      prevLiquidityPool.accumulatedProtocolFeeB + protocolFeeAmount
    );
  });

  it("`swap` method using 'exact out' param with mint B!", async () => {
    const poolId = 0;
    const OUTPUT_AMOUNT = 2_000_000; // 2 token

    const param: anchor.IdlTypes<Amm>["swapParams"] = {
      exactOut: { outputAmount: bn(OUTPUT_AMOUNT) },
    };

    const [prevLiquidityPool] = await getLiquidityPoolAccount(poolId);
    console.log("A amount in vault:", prevLiquidityPool.amountMintA);
    console.log("B amount in vault:", prevLiquidityPool.amountMintB);

    const tx = await program.methods
      .swap(bn(poolId), param)
      .accounts({
        signer: user.publicKey,
        inputMint: mintB,
        outputMint: mintA,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([user])
      .rpc();

    console.log("`swap` tx signature:", tx);

    const { inputAmount, outputAmount, protocolFeeAmount } = calculateSwapAmounts(
      param,
      prevLiquidityPool.amountMintB,
      prevLiquidityPool.amountMintA,
      prevLiquidityPool.feeBps,
      prevLiquidityPool.protocolFeeBps
    );

    console.log({ inputAmount, outputAmount, protocolFeeAmount });

    const [liquidityPool] = await getLiquidityPoolAccount(poolId);

    expect(liquidityPool.amountMintB, "amount mint B").eq(
      prevLiquidityPool.amountMintB + inputAmount - protocolFeeAmount
    );
    expect(liquidityPool.amountMintA, "amount mint A").eq(
      prevLiquidityPool.amountMintA - outputAmount
    );
    expect(liquidityPool.accumulatedProtocolFeeB, "accumulated fee B").eq(
      prevLiquidityPool.accumulatedProtocolFeeB + protocolFeeAmount
    );
  });

  it("`withdraw_protocol_fees` method!", async () => {
    const poolId = 0;

    const tx = await program.methods
      .withdrawProtocolFees(bn(poolId))
      .accounts({ tokenProgram: TOKEN_PROGRAM_ID })
      .rpc();

    console.log("`withdraw_protocol_fees` tx signature:", tx);

    const [liquidityPool] = await getLiquidityPoolAccount(poolId);
    expect(liquidityPool.accumulatedProtocolFeeA).eq(0);
    expect(liquidityPool.accumulatedProtocolFeeB).eq(0);
  });
});
