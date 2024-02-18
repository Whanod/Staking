import * as anchor from "@project-serum/anchor";
import { utils, BN } from "@project-serum/anchor";
import { Ed25519Keypair, Keypair, PublicKey } from "@solana/web3.js";
import { Program } from "@project-serum/anchor";
import { expect } from "chai";
import * as token from "@solana/spl-token";
import { Staking } from "../target/types/staking";
import { NftData, createNft, upload_metdata } from "./nft";
import { readFileSync } from "fs";
import {
  FindNftByMintInput,
  Metaplex,
  Nft,
  Signer,
  SplTokenAmount,
  WalletAdapterIdentityDriver,
  createTokenWithMintOperation,
  keypairIdentity,
  mockStorage,
  toBigNumber,
  toMetaplexFile,
  walletAdapterIdentity,
} from "@metaplex-foundation/js";
import { associatedAddress } from "@project-serum/anchor/dist/cjs/utils/token";
import { findProgramAddressSync } from "@project-serum/anchor/dist/cjs/utils/pubkey";

describe("anchor-counter", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.local();
  anchor.setProvider(provider);

  const meta = Metaplex.make(provider.connection)
    .use(walletAdapterIdentity(provider.wallet))
    .use(mockStorage());
  const program = anchor.workspace.Staking as Program<Staking>;
  const programId = new PublicKey(
    "ATfdE39GhVCzGEeX8kVnbPwb1Uur7fBX8jCU1SrL3Swq"
  );

  it("Is initialized!", async () => {
    const program = anchor.workspace.Staking as Program<Staking>;

    const nftData = {
      name: "Name",
      symbol: "SYMBOL",
      description: "Description",
      sellerFeeBasisPoints: 100,
      imageFile: "solana.png",
    };
    let collection_address: PublicKey;
    let token_mint: PublicKey;
    let token_account: PublicKey;
    let nft_mint: PublicKey;
    let nft_token: PublicKey;
    let nft_edition: PublicKey;
    let nft_metadata: PublicKey;

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
      },
      {
        commitment: "finalized",
      }
    );

    let out = await meta.nfts().verifyCollection(
      {
        mintAddress: nft.mint.address,
        collectionMintAddress: collection.mintAddress,
        isSizedCollection: true,
      },
      { commitment: "finalized" }
    );
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

    let { token } = await meta.tokens().createTokenWithMint({
      mintAuthority: provider.wallet as Signer,
      decimals: 1,
      initialSupply: {
        basisPoints: toBigNumber(100000),
        currency: {
          symbol: "FUR",
          decimals: 1,
          namespace: "spl-token",
        },
      },
    });
    token_mint = token.mint.address;
    token_account = token.address;

    const [stake_details] = findProgramAddressSync(
      [
        utils.bytes.utf8.encode("stake"),
        collection_address.toBytes(),
        provider.publicKey.toBytes(),
      ],
      programId
    );
    const [token_authority] = findProgramAddressSync(
      [utils.bytes.utf8.encode("token-authority"), stake_details.toBytes()],
      programId
    );
    const [nft_authority] = PublicKey.findProgramAddressSync(
      [utils.bytes.utf8.encode("nft-authority"), stake_details.toBytes()],
      programId
    );

    let tx = await program.methods
      .init(new BN(500))
      .accounts({
        stakeDetails: stake_details,
        tokenMint: token_mint,
        collectionAddress: collection_address,
        nftAuthority: nft_authority,
        tokenAuthority: token_authority,
      })
      .rpc();
    console.log(tx);
  });

  it("Stake NFT", async () => {});
});
