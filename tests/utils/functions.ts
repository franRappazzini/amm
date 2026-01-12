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
