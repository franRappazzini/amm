import * as anchor from "@coral-xyz/anchor";

import { Amm } from "../target/types/amm";
import { Program } from "@coral-xyz/anchor";

describe("amm", () => {
  const provider = anchor.AnchorProvider.env();
  const { connection, wallet } = provider;

  anchor.setProvider(provider);

  const program = anchor.workspace.amm as Program<Amm>;

  it("`initialize` method!", async () => {
    const FEE_BPS = 30; // 0.3%

    const tx = await program.methods.initialize(FEE_BPS).rpc();
    console.log("`initialize` tx signature:", tx);
  });
});
