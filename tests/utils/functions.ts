import { BN } from "bn.js";

export function bn(num: number) {
  return new BN(num);
}

/// qM = √(a*b) - 1000
export function initialMintLiquidity(amount_a: number, amount_b: number): number {
  return Math.floor(Math.sqrt(amount_a * amount_b)) - 1000;
}
