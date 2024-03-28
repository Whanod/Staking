use crate::state::stake_details::Deatils;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{
        mpl_token_metadata::types::AuthorizationData, MasterEditionAccount, Metadata,
        MetadataAccount,
    },
    token::{Mint, Token, TokenAccount},
};
use mpl_token_metadata::instructions::TransferV1CpiBuilder;

use crate::state::stake_details;
#[inline(never)]
#[allow(clippy::too_many_arguments)]
pub fn send_pnft<'info>(
    metadata_program: &Program<'info, Metadata>,
    authority: &UncheckedAccount<'info>,
    owner: &UncheckedAccount<'info>,
    payer: &UncheckedAccount<'info>,
    nft_source: &Account<'info, TokenAccount>,
    nft_dest: &Account<'info, TokenAccount>,
    dest_authority: &AccountInfo<'info>,
    nft_mint: &Account<'info, Mint>,
    nft_metadata: &Account<'info, MetadataAccount>,
    edition: &Account<'info, MasterEditionAccount>,
    system_program: &Program<'info, System>,
    token_program: &Program<'info, Token>,
    ata_program: &Program<'info, AssociatedToken>,
    instructions: &AccountInfo<'info>,
    owner_token_record: &AccountInfo<'info>,
    dest_token_record: &AccountInfo<'info>,
    authorization_rules_program: &AccountInfo<'info>,
    rules: &AccountInfo<'info>,
    is_pda_signer: bool,
    stake_details: &Account<'info, Deatils>,
) -> Result<()> {
    let payer_info = payer.to_account_info();
    let metadata_program_info = metadata_program.to_account_info();
    let nft_token_info = nft_source.to_account_info();
    let edition_account_info = edition.to_account_info();
    let metadata_account_info = nft_metadata.to_account_info();
    let nft_custody_info = nft_dest.to_account_info();
    let nft_authority_info = dest_authority.to_account_info();
    let nft_mint_info = nft_mint.to_account_info();
    let auth_program_info = authorization_rules_program.to_account_info();
    let auth_rules_info = rules.to_account_info();
    let token_record_info = owner_token_record.to_account_info();
    let dest_token_record_info = dest_token_record.to_account_info();
    let mut builder = TransferV1CpiBuilder::new(metadata_program);
    builder
        .token(&nft_token_info)
        .mint(&nft_mint_info)
        .token_owner(&owner)
        .spl_token_program(&token_program)
        .spl_ata_program(&ata_program)
        .authority(&authority)
        .edition(Some(&edition_account_info))
        .metadata(&metadata_account_info)
        .payer(&payer)
        .sysvar_instructions(&instructions)
        .system_program(&system_program)
        .destination_token(&nft_custody_info)
        .destination_owner(&nft_authority_info)
        .authorization_rules_program(Some(&auth_program_info))
        .authorization_rules(Some(&auth_rules_info))
        .token_record(Some(&token_record_info))
        .destination_token_record(Some(&dest_token_record_info))
        .amount(1);
    if is_pda_signer {
        let key = &stake_details.key();
        let auth_seeds = &[
            &b"nft-authority"[..],
            &key.as_ref(),
            &[stake_details.nft_auth_bump],
        ];
        builder.invoke_signed(&[&auth_seeds[..]])?;
    } else {
        builder.invoke()?;
    }

    Ok(())
}
