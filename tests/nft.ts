import {
  Metaplex,
  Nft,
  NftWithToken,
  Signer,
  toMetaplexFile,
} from "@metaplex-foundation/js";
import { readFileSync } from "fs";

export interface NftData {
  name: string;
  symbol: string;
  description: string;
  sellerFeeBasisPoints: number;
  imageFile: string;
}

interface CollectionNftData {
  name: string;
  symbol: string;
  description: string;
  sellerFeeBasisPoints: number;
  imageFile: string;
  isCollection: boolean;
  collectionAuthority: Signer;
}

// example data for a new NFT

// example data for updating an existing NFT
const updateNftData = {
  name: "Update",
  symbol: "UPDATE",
  description: "Update Description",
  sellerFeeBasisPoints: 100,
  imageFile: "success.png",
};

// helper function create NFT
export async function createNft(
  metaplex: Metaplex,
  uri: string,
  nftData: NftData
): Promise<NftWithToken> {
  const { nft } = await metaplex.nfts().create(
    {
      uri: uri, // metadata URI
      name: nftData.name,
      sellerFeeBasisPoints: nftData.sellerFeeBasisPoints,
      symbol: nftData.symbol,
    },
    { commitment: "finalized" }
  );

  return nft as NftWithToken;
}

export async function upload_metdata(
  metaplex: Metaplex,
  nft_data: NftData
): Promise<string> {
  const buffer = readFileSync(
    "/home/whanod/projects/rust/staking/tests/" + nft_data.imageFile
  );

  // buffer to metaplex file

  const file = toMetaplexFile(buffer, nft_data.imageFile);

  // upload image and get image uri

  const imageUri = await metaplex.storage().upload(file);

  // upload metadata and get metadata uri (off chain metadata)

  const { uri } = await metaplex.nfts().uploadMetadata({
    name: nft_data.name,

    symbol: nft_data.symbol,

    description: nft_data.description,

    image: imageUri,
  });

  return uri;
}
