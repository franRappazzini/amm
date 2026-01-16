import * as anchor from "@coral-xyz/anchor";

import { GLOBAL_CONFIG_SEED, LIQUIDITY_POOL_SEED } from "./constants";
import { globalConfigParser, liquidityPoolParser } from "./parsers";

import { Amm } from "../../target/types/amm";
import { PublicKey } from "@solana/web3.js";
import { bn } from "./functions";

const program = anchor.workspace.amm as anchor.Program<Amm>;

const getGlobalConfigAccount = async () => {
  const [globalConfigPda] = PublicKey.findProgramAddressSync(
    [GLOBAL_CONFIG_SEED],
    anchor.workspace.amm.programId,
  );

  return [
    globalConfigParser(await program.account.globalConfig.fetchNullable(globalConfigPda)),
    globalConfigPda,
  ] as const;
};

const getLiquidityPoolAccount = async (id: number) => {
  const [liquidityPoolPda] = PublicKey.findProgramAddressSync(
    [LIQUIDITY_POOL_SEED, bn(id).toArrayLike(Buffer, "le", 8)],
    anchor.workspace.amm.programId,
  );

  return [
    liquidityPoolParser(await program.account.liquidityPool.fetch(liquidityPoolPda)),
    liquidityPoolPda,
  ] as const;
};

export { getGlobalConfigAccount, getLiquidityPoolAccount };
