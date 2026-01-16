import * as anchor from "@coral-xyz/anchor";

import { Amm } from "../../target/types/amm";

interface ParsedGlobalConfig {
  authority: string;
  poolCount: number;
  bump: number;
  feeBps: number;
}

const globalConfigParser = (
  data: anchor.IdlAccounts<Amm>["globalConfig"] | null,
): ParsedGlobalConfig | null => {
  if (data === null) return null;
  return {
    ...data,
    authority: data.authority.toBase58(),
    poolCount: data.poolCount.toNumber(),
  };
};

interface ParsedLiquidityPool {
  creator: string;
  bump: number;
  mintA: string;
  mintB: string;
  lpMint: string;
  feeBps: number;
  protocolFeeBps: number;
  lpSupply: number;
  amountMintA: number;
  amountMintB: number;
  accumulatedProtocolFeeA: number;
  accumulatedProtocolFeeB: number;
}

const liquidityPoolParser = (
  data: anchor.IdlAccounts<Amm>["liquidityPool"],
): ParsedLiquidityPool => {
  return {
    ...data,
    creator: data.creator.toBase58(),
    mintA: data.mintA.toBase58(),
    mintB: data.mintB.toBase58(),
    lpMint: data.lpMint.toBase58(),
    lpSupply: data.lpSupply.toNumber(),
    amountMintA: data.amountMintA.toNumber(),
    amountMintB: data.amountMintB.toNumber(),
    accumulatedProtocolFeeA: data.accumulatedProtocolFeeA.toNumber(),
    accumulatedProtocolFeeB: data.accumulatedProtocolFeeB.toNumber(),
  };
};

export { globalConfigParser, liquidityPoolParser };
