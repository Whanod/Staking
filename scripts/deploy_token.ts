import {
  Connection,
  Keypair,
  PublicKey,
  Transaction,
  clusterApiUrl,
  sendAndConfirmTransaction,
} from "@solana/web3.js";

import "dotenv/config";
import { Metaplex } from "@metaplex-foundation/js";
import { getKeypairFromEnvironment } from "@solana-developers/node-helpers";
import {
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
} from "@solana/spl-token";
import { createCreateMetadataAccountV3Instruction } from "@metaplex-foundation/mpl-token-metadata";
import { publicKey } from "@project-serum/anchor/dist/cjs/utils";
const main = async () => {
  let senderKey = getKeypairFromEnvironment("MAIN");
  const connection = new Connection(clusterApiUrl("mainnet-beta"), {
    commitment: "confirmed",
  });
  const metaplex = new Metaplex(connection);
  const mintAccount = await createMint(
    connection,
    senderKey,
    senderKey.publicKey,
    senderKey.publicKey,
    6
  );

  const metadata = metaplex.nfts().pdas().metadata({ mint: mintAccount });
  const tx = new Transaction().add(
    createCreateMetadataAccountV3Instruction(
      {
        metadata: metadata,
        mint: mintAccount,
        mintAuthority: senderKey.publicKey,
        payer: senderKey.publicKey,
        updateAuthority: senderKey.publicKey,
      },
      {
        createMetadataAccountArgsV3: {
          data: {
            name: "Fur",
            symbol: "Fur",
            uri: "https://bafkreifh4feyttismhzlhdptelpxcqj64rc22bfkhg6pgegvz2o6hty57u.ipfs.nftstorage.link/",
            sellerFeeBasisPoints: 0,
            collection: null,
            creators: null,
            uses: null,
          },
          isMutable: true,
          collectionDetails: null,
        },
      }
    )
  );
  await sendAndConfirmTransaction(connection, tx, [senderKey]);
  const tokenAccount = await getOrCreateAssociatedTokenAccount(
    connection,
    senderKey,
    mintAccount,
    senderKey.publicKey
  );
  let decimals = 1_000_000;
  await mintTo(
    connection,
    senderKey,
    mintAccount,
    tokenAccount.address,
    senderKey,
    100_000_000 * decimals
  );
  console.log(mintAccount);
};

main();
