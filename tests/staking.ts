import * as anchor from "@project-serum/anchor";
import { utils, BN } from "@project-serum/anchor";
import { PublicKey } from "@solana/web3.js";
import { Program } from "@project-serum/anchor";
import { expect } from "chai";
import * as token from "@solana/spl-token";
import { NftStake } from "../target/types/nft_stake";

describe("anchor-counter", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Staking as Program<NftStake>;

  const counter = anchor.web3.Keypair.generate();

  it("Is initialized!", async () => {
    // Add your test here.
  });

  it("Stake NFT", async () => {});
});
