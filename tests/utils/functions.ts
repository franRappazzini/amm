import { Amm } from "../../target/types/amm";
import { BN } from "bn.js";
import { IdlTypes } from "@coral-xyz/anchor";

export function bn(num: number) {
  return new BN(num);
}

/// qM = √(a*b) - 1000
export function initialMintLiquidity(amount_a: number, amount_b: number): number {
  return Math.floor(Math.sqrt(amount_a * amount_b)) - 1000;
}

// qM = min { (Cx / A) * M , (Cy / B) * M }
export function subsequentMintLiquidity(
  amountA: number,
  amountB: number,
  supply: number,
  reserveA: number,
  reserveB: number
): number {
  const liquidityA = Math.floor((amountA * supply) / reserveA);
  const liquidityB = Math.floor((amountB * supply) / reserveB);

  return Math.min(liquidityA, liquidityB);
}

export function calculateDepositExcess(
  amountA: number,
  amountB: number,
  reserveA: number,
  reserveB: number
): { newAmountA: number; newAmountB: number } {
  const deposited = Math.floor(amountB / amountA);
  const reserves = Math.floor(reserveB / reserveA);

  let newAmountA = amountA;
  let newAmountB = amountB;

  if (deposited > reserves) {
    const res = Math.floor((reserveB * amountA) / reserveA);
    const excessB = amountB - res;
    newAmountB = amountB - excessB;
  } else if (deposited < reserves) {
    const res = Math.floor((reserveA * amountB) / reserveB);
    const excessA = amountA - res;
    newAmountA = amountA - excessA;
  }

  return { newAmountA, newAmountB };
}

// (m / M) * T)
export function calculateClaimableAmount(
  lpAmount: number,
  lpSupply: number,
  mintReserve: number
): number {
  return Math.floor((lpAmount / lpSupply) * mintReserve);
}

// isolating a - if the trader deposits an amount `b` of token Y, they will receive an amount `a` of token X
// a = A * (1 - ϕ)*b / (B + (1 - ϕ)*b)
// isolating b - in order to receive an amount `a` of token X, the trader must deposit an amount `b` of token Y
// b = aB / (A - a) * (1 - ϕ)
export function calculateSwapAmounts(
  swapParams: IdlTypes<Amm>["swapParams"],
  reserveInput: number,
  reserveOutput: number,
  feeBps: number
): { inputAmount: number; outputAmount: number } {
  const feeMultiplier = 10000 - feeBps;

  if ("exactIn" in swapParams) {
    const inputAmount = swapParams.exactIn.inputAmount.toNumber();
    if (inputAmount <= 0) {
      throw new Error("InsufficientInputAmount");
    }

    const inputAmountAfterFee = Math.floor((inputAmount * feeMultiplier) / 10000);

    const numerator = reserveOutput * inputAmountAfterFee;
    const denominator = reserveInput + inputAmountAfterFee;

    const outputAmount = Math.floor(numerator / denominator);

    return { inputAmount, outputAmount };
  } else {
    const outputAmount = swapParams.exactOut.outputAmount.toNumber();
    if (outputAmount <= 0) {
      throw new Error("InsufficientInputAmount");
    }

    const numerator = outputAmount * reserveInput;
    const denominator = Math.floor(((reserveOutput - outputAmount) * feeMultiplier) / 10000);

    const inputAmount = Math.floor(numerator / denominator);

    return { inputAmount, outputAmount };
  }
}
