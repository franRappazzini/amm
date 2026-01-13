import { BN } from "bn.js";

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
