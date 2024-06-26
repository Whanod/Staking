import * as anchor from "@project-serum/anchor";
import { exec } from "child_process";
import { utils, BN } from "@project-serum/anchor";
import {
  ComputeBudgetProgram,
  Keypair,
  PUBLIC_KEY_LENGTH,
  PublicKey,
  SYSVAR_INSTRUCTIONS_PUBKEY,
  SYSVAR_RENT_PUBKEY,
  Transaction,
} from "@solana/web3.js";
import { Program } from "@project-serum/anchor";

import * as spl from "@solana/spl-token";
import { Staking } from "../target/types/staking";
import { upload_metdata } from "./nft";
import "dotenv/config";
import { getKeypairFromEnvironment } from "@solana-developers/node-helpers";

import {
  Metaplex,
  Nft,
  Signer,
  ThawDelegatedNftInput,
  amount,
  mockStorage,
  toBigNumber,
  walletAdapterIdentity,
} from "@metaplex-foundation/js";
import { associatedAddress } from "@project-serum/anchor/dist/cjs/utils/token";
import { findProgramAddressSync } from "@project-serum/anchor/dist/cjs/utils/pubkey";
import { expect } from "chai";
import {
  AuthorizationTokenAccountOwnerMismatchError,
  TokenRecord,
  TokenStandard,
  ruleSetToggleBeet,
} from "@metaplex-foundation/mpl-token-metadata";
import { publicKey } from "@coral-xyz/anchor/dist/cjs/utils";

describe("anchor-staking-nft", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.local();
  anchor.setProvider(provider);
  const ownerWallet = getKeypairFromEnvironment("SECRET_KEY");

  const meta = Metaplex.make(provider.connection)
    .use(walletAdapterIdentity(provider.wallet))
    .use(mockStorage());
  const program = anchor.workspace.Staking as Program<Staking>;
  const programId = new PublicKey(
    "ATfdE39GhVCzGEeX8kVnbPwb1Uur7fBX8jCU1SrL3Swq"
  );
  let collection_address: PublicKey;
  let token_mint: PublicKey;
  let token_account: PublicKey;
  let nft_mint: PublicKey;
  let nft_token: PublicKey;
  let nft_edition: PublicKey;
  let nft_metadata: PublicKey;
  let stake_details;
  let token_authority;
  let nft_authority;
  let staking_record;
  let nft_custody;
  let token;
  let user_token_address;
  let decimals = 1_000_000;
  let pnft: Nft;
  let rules = new PublicKey("AXGujE6T556PjoN8yXpN74hQj8uB9B9YDNyCcryLeizW");
  let token_record: PublicKey;
  let token_record_dest: PublicKey;
  it("Is initialized!", async () => {
    const program = anchor.workspace.Staking as Program<Staking>;

    const nftData = {
      name: "Name",
      symbol: "SYMBOL",
      description: "Description",
      sellerFeeBasisPoints: 100,
      imageFile: "solana.png",
    };

    const uri = await upload_metdata(meta, nftData);
    const collection = await meta.nfts().create(
      {
        uri: uri,
        name: "fr",
        sellerFeeBasisPoints: 30,
        isCollection: true,
      },
      {
        commitment: "finalized",
      }
    );
    const { nft } = await meta.nfts().create(
      {
        uri: uri,
        name: "fr1",
        sellerFeeBasisPoints: 30,
        collection: collection.mintAddress,
        tokenStandard: TokenStandard.ProgrammableNonFungible,
        ruleSet: rules,
      },

      {
        commitment: "finalized",
      }
    );
    pnft = nft;

    let out = await meta.nfts().verifyCollection(
      {
        mintAddress: nft.mint.address,
        collectionMintAddress: collection.mintAddress,
        isSizedCollection: true,
      },
      { commitment: "finalized" }
    );
    console.log(nft.edition.address);
    let verified_nft = await meta
      .nfts()
      .findByMint({ mintAddress: nft.mint.address });

    nft_mint = verified_nft.address;
    nft_metadata = verified_nft.metadataAddress;
    nft_token = await associatedAddress({
      mint: nft_mint,
      owner: provider.wallet.publicKey,
    });

    nft_edition = nft.edition.address;
    collection_address = collection.mintAddress;

    let output = await meta.tokens().createTokenWithMint({
      mintAuthority: provider.wallet as Signer,
      decimals: 6,
      initialSupply: {
        basisPoints: toBigNumber(0),
        currency: {
          symbol: "FUR",
          decimals: 6,
          namespace: "spl-token",
        },
      },
    });

    token_mint = output.token.mint.address;
    token_account = output.token.address;
    user_token_address = await associatedAddress({
      mint: token_mint,
      owner: provider.publicKey,
    });
    let user_token_account = await spl.getAccount(
      provider.connection,
      user_token_address
    );
    console.log("token amount", user_token_account.amount.toString());

    [stake_details] = findProgramAddressSync(
      [
        utils.bytes.utf8.encode("stake"),
        collection_address.toBytes(),
        provider.publicKey.toBytes(),
      ],
      programId
    );
    [token_authority] = findProgramAddressSync(
      [utils.bytes.utf8.encode("token-authority"), stake_details.toBytes()],
      programId
    );
    let [freeze] = findProgramAddressSync(
      [
        utils.bytes.utf8.encode("metadata"),
        new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s").toBytes(),
        nft_mint.toBytes(),
        utils.bytes.utf8.encode("edition"),
      ],
      new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s")
    );

    [nft_authority] = PublicKey.findProgramAddressSync(
      [utils.bytes.utf8.encode("nft-authority"), stake_details.toBytes()],
      programId
    );
    nft_custody = await spl.getAssociatedTokenAddress(
      nft_mint,
      nft_authority,
      true
    );
    [staking_record] = PublicKey.findProgramAddressSync(
      [
        utils.bytes.utf8.encode("staking-record"),
        stake_details.toBytes(),
        nft_mint.toBytes(),
      ],
      programId
    );
    [token_record] = findProgramAddressSync(
      [
        utils.bytes.utf8.encode("metadata"),
        new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s").toBytes(),
        nft_mint.toBytes(),
        utils.bytes.utf8.encode("token_record"),
        nft_token.toBytes(),
      ],
      new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s")
    );
    [token_record_dest] = findProgramAddressSync(
      [
        utils.bytes.utf8.encode("metadata"),
        new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s").toBytes(),
        nft_mint.toBytes(),
        utils.bytes.utf8.encode("token_record"),
        nft_custody.toBytes(),
      ],
      new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s")
    );

    console.log();

    let tx = await program.methods
      .init(new BN(50))
      .accounts({
        stakeDetails: stake_details,
        tokenMint: token_mint,
        collectionAddress: collection_address,
        nftAuthority: nft_authority,
        tokenAuthority: token_authority,
      })
      .rpc({ commitment: "confirmed" });
  });

  it("Stake NFT", async () => {
    const tx = await program.methods
      .stake(1)
      .accounts({
        staker: provider.publicKey,
        metadataProgram: new PublicKey(
          "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
        ),
        authProgram: new PublicKey(
          "auth9SigNpDKz4sJJ1DfCTuZrZNSAgh9sFD3rboVmgg"
        ),

        tokenRecord: token_record,
        tokenRecordDest: token_record_dest,
        authRules: pnft.programmableConfig.ruleSet,
        sysvarInstructions: new PublicKey(
          "Sysvar1nstructions1111111111111111111111111"
        ),
        stakeDetails: stake_details,
        nftAuthority: nft_authority,
        stakingRecord: staking_record,
        nftMint: nft_mint,
        nftEdition: nft_edition,
        nftMetadata: nft_metadata,
        nftToken: nft_token,
        nftCustody: nft_custody,
      })
      .instruction();
    let tr = new Transaction();

    tr.add(ComputeBudgetProgram.setComputeUnitLimit({ units: 300000 }));
    tr.add(tx);

    let res = await provider
      .sendAndConfirm(tr, [], { commitment: "confirmed" })
      .catch((err) => console.error(err));
    if (res) {
      let cnf_output = await meta
        .nfts()
        .findAllByOwner({ owner: nft_authority });
      console.log("found:", cnf_output[0].address);

      let parsed_tx = await provider.connection.getParsedTransaction(res, {
        commitment: "confirmed",
      });
      console.log(parsed_tx.blockTime);
    }
  });
  const delay = (ms: number) => new Promise((res) => setTimeout(res, ms));

  it("Claim Reward", async () => {
    await delay(1000);

    const tx = await program.methods
      .claim()
      .accounts({
        stakeDetails: stake_details,
        stakingRecord: staking_record,
        rewardMint: token_mint,
        rewardReceiveAccount: token_account,
        tokenAuthority: token_authority,
      })
      .rpc({ commitment: "confirmed" });
    let parsed_tx = await provider.connection.getParsedTransaction(tx, {
      commitment: "confirmed",
    });

    user_token_address = await associatedAddress({
      mint: token_mint,
      owner: provider.publicKey,
    });
    let user_token_account = await spl.getAccount(
      provider.connection,
      user_token_address
    );
    console.log(parsed_tx.blockTime);

    console.log("token amount", Number(user_token_account.amount) / decimals);
  });
  it("Unstake NFT", async () => {
    let tx = await program.methods
      .unstake()
      .accounts({
        nftEdition: nft_edition,
        nftMetadata: nft_metadata,
        authRules: pnft.programmableConfig.ruleSet,
        metadataProgram: new PublicKey(
          "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
        ),
        authProgram: new PublicKey(
          "auth9SigNpDKz4sJJ1DfCTuZrZNSAgh9sFD3rboVmgg"
        ),
        sysvarInstructions: new PublicKey(
          "Sysvar1nstructions1111111111111111111111111"
        ),
        tokenRecord: token_record_dest,
        tokenRecordDest: token_record,
        stakeDetails: stake_details,
        stakingRecord: staking_record,
        rewardMint: token_mint,
        rewardReceiveAccount: user_token_address,
        tokenAuthority: token_authority,
        nftAuthority: nft_authority,
        nftCustody: nft_custody,
        nftMint: nft_mint,
        nftReceiveAccount: nft_token,
      })
      .instruction();
    let tr = new Transaction();

    tr.add(ComputeBudgetProgram.setComputeUnitLimit({ units: 300000 }));
    tr.add(tx);
    provider
      .sendAndConfirm(tr, [], { commitment: "confirmed" })
      .catch((err) => {
        console.log("nft_authority: ", nft_authority);
        console.log("signer: ", provider.publicKey);
        console.error(err);
      });
  });

  it("Closes Staking", async () => {
    const tx = await program.methods
      .closeStaking()
      .accounts({
        stakeDetails: stake_details,
        tokenMint: token_mint,
        tokenAuthority: token_authority,
      })
      .rpc()
      .catch((err) => {
        console.error(err);
      });
  });
});
