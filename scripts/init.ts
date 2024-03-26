import { Connection, Keypair, clusterApiUrl } from "@solana/web3.js";

import "dotenv/config";
import { PublicKey } from "@solana/web3.js";
import { utils } from "@project-serum/anchor";
import { findProgramAddressSync } from "@project-serum/anchor/dist/cjs/utils/pubkey";
import { getKeypairFromEnvironment } from "@solana-developers/node-helpers";
import { AnchorProvider, Wallet, Program } from "@coral-xyz/anchor";
import { IDL } from "../app/src/idl/staking";
import { BN } from "bn.js";

const main = async () => {
  const connection = new Connection(clusterApiUrl("mainnet-beta"), {
    commitment: "confirmed",
  });

  const tokenMintAccount = process.env.TOKEN_MINT_ACCOUNT;

  const TokenOwner = getKeypairFromEnvironment("MAIN");
  const wallet = new Wallet(TokenOwner);
  const collectionAddress = new PublicKey(process.env.COLLECTION_ADDRESS);
  const provider = new AnchorProvider(connection, wallet, {});

  const programId = new PublicKey(process.env.PROGRAM_ID);
  const program = new Program(IDL, programId, provider);
  let [stake_details] = findProgramAddressSync(
    [
      utils.bytes.utf8.encode("stake"),
      collectionAddress.toBytes(),
      TokenOwner.publicKey.toBytes(),
    ],
    programId
  );
  let [token_authority] = findProgramAddressSync(
    [utils.bytes.utf8.encode("token-authority"), stake_details.toBytes()],
    programId
  );
  let [nft_authority] = PublicKey.findProgramAddressSync(
    [utils.bytes.utf8.encode("nft-authority"), stake_details.toBytes()],
    programId
  );

  let tx = await program.methods
    .init(new BN(50))
    .accounts({
      stakeDetails: stake_details,
      tokenMint: tokenMintAccount,
      collectionAddress: collectionAddress,
      nftAuthority: nft_authority,
      tokenAuthority: token_authority,
    })
    .rpc({ commitment: "confirmed" });
  console.log(stake_details);
  console.log(tx);
};
main();
