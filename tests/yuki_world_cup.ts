import * as anchor from "@anchor-lang/core";
import { Program } from "@anchor-lang/core";
import { YukiWorldCup } from "../target/types/yuki_world_cup";

describe("yuki_world_cup", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.yukiWorldCup as Program<YukiWorldCup>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
