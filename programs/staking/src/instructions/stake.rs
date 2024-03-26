use crate::state::{events::StakeEvent, stake_details::Deatils, staking_record::StakingRecord};
use crate::StakeError;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::metadata::{MasterEditionAccount, TokenRecordAccount};
use anchor_spl::metadata::{Metadata, MetadataAccount};
use anchor_spl::token::{Mint, Token, TokenAccount};

use mpl_token_metadata::instructions::TransferV1CpiBuilder;

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(seeds=[b"stake",stake_details.collection.as_ref(),stake_details.creator.as_ref()],bump=stake_details.stake_bump)]
    pub stake_details: Box<Account<'info, Deatils>>,
    #[account(init,payer=staker,space=StakingRecord::LEN+8,seeds=[b"staking-record",stake_details.key().as_ref(),nft_mint.key().as_ref()],bump)]
    pub staking_record: Box<Account<'info, StakingRecord>>,
    pub system_program: Program<'info, System>,
    #[account(mut)]
    pub staker: Signer<'info>,
    #[account(mut,mint::decimals = 0,constraint= nft_mint.supply == 1 @ StakeError::TokenNotNFT )]
    pub nft_mint: Box<Account<'info, Mint>>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    #[account(mut,associated_token::mint= nft_mint, associated_token::authority = staker,constraint= nft_token.amount ==1 @ StakeError::TokenAccountEmpty)]
    pub nft_token: Box<Account<'info, TokenAccount>>,

    pub metadata_program: Program<'info, Metadata>,
    /// CHECK: Sysvar
    pub sysvar_instructions: AccountInfo<'info>,

    #[account(mut,
        seeds = [
            b"metadata",
            Metadata::id().as_ref(),
            nft_mint.key().as_ref()
        ],
        seeds::program = Metadata::id(),
        bump,
        constraint = nft_metadata.collection.as_ref().unwrap().verified @ StakeError::CollectionNotVerified,
        constraint = nft_metadata.collection.as_ref().unwrap().key == stake_details.collection @ StakeError::InvalidCollection
    )]
    nft_metadata: Box<Account<'info, MetadataAccount>>,
    #[account(seeds=[b"metadata",Metadata::id().as_ref(),nft_mint.key().as_ref(),b"edition"], seeds::program = Metadata::id(),bump)]
    nft_edition: Box<Account<'info, MasterEditionAccount>>,

    #[account(
        seeds = [
            b"nft-authority",
            stake_details.key().as_ref()
        ],
        bump = stake_details.nft_auth_bump
    )]
    /// CHECK: This account is not read or written
    pub nft_authority: UncheckedAccount<'info>,
    #[account(
        init,
        payer = staker,
        associated_token::mint = nft_mint,
        associated_token::authority = nft_authority
    )]
    pub nft_custody: Box<Account<'info, TokenAccount>>,
    /// CHECK: Sysvar
    pub auth_program: AccountInfo<'info>,
    /// CHECK: Sysvar
    pub auth_rules: AccountInfo<'info>,
    #[account(
        seeds = [
            b"metadata",Metadata::id().as_ref(),nft_mint.key().as_ref(),b"token_record",nft_token.key().as_ref()
        ],seeds::program=Metadata::id(),bump
    )]
    pub token_record: Box<Account<'info, TokenRecordAccount>>,
    #[account(
        seeds = [
            b"metadata",Metadata::id().as_ref(),nft_mint.key().as_ref(),b"token_record",nft_custody.key().as_ref()
        ],seeds::program=Metadata::id(),bump
    )]
    /// CHECK: Sysvar
    pub token_record_dest: AccountInfo<'info>,
}

pub fn stake_handler(ctx: Context<Stake>, staking_period: u8) -> Result<()> {
    let periods: Vec<u8> = vec![1, 3, 6, 12];
    require_eq!(
        periods.contains(&staking_period),
        true,
        StakeError::StakePeriodError
    );
    let clock = clock::Clock::get()?;
    let current_time = clock.unix_timestamp;

    let staking_status = ctx.accounts.stake_details.is_active;
    require_eq!(staking_status, true, StakeError::StakingInactive);

    let user = ctx.accounts.staker.key();

    let nft_mint_key = ctx.accounts.nft_mint.key();
    let id = Metadata::id();
    let auth_seeds = &[
        &b"metadata"[..],
        &id.as_ref(),
        &nft_mint_key.as_ref(),
        &b"edition"[..],
        &[ctx.bumps.nft_edition],
    ];
    let metadata_program_info = ctx.accounts.metadata_program.to_account_info();
    let nft_token_info = ctx.accounts.metadata_program.to_account_info();
    let edition_account_info = ctx.accounts.nft_edition.to_account_info();
    let metadata_account_info = ctx.accounts.nft_edition.to_account_info();
    let nft_custody_info = ctx.accounts.nft_custody.to_account_info();
    let nft_authority_info = ctx.accounts.nft_authority.to_account_info();
    let nft_mint_info = ctx.accounts.nft_mint.to_account_info();
    let auth_program_info = ctx.accounts.auth_program.to_account_info();
    let auth_rules_info = ctx.accounts.auth_rules.to_account_info();
    let token_record_info = ctx.accounts.token_record.to_account_info();
    let dest_token_record_info = ctx.accounts.token_record_dest.to_account_info();

    let bump = ctx.bumps.staking_record;
    let mut builder = TransferV1CpiBuilder::new(&metadata_program_info);
    builder
        .token(&nft_token_info)
        .mint(&nft_mint_info)
        .token_owner(&ctx.accounts.staker)
        .spl_token_program(&ctx.accounts.token_program)
        .spl_ata_program(&ctx.accounts.associated_token_program)
        .authority(&ctx.accounts.staker)
        .edition(Some(&edition_account_info))
        .metadata(&metadata_account_info)
        .payer(&ctx.accounts.staker)
        .sysvar_instructions(&ctx.accounts.sysvar_instructions)
        .system_program(&ctx.accounts.system_program)
        .destination_token(&nft_custody_info)
        .destination_owner(&nft_authority_info)
        .authorization_rules_program(Some(&auth_program_info))
        .authorization_rules(Some(&auth_rules_info))
        .token_record(Some(&token_record_info))
        .destination_token_record(Some(&dest_token_record_info))
        .amount(1);

    builder.invoke_signed(&[&auth_seeds[..]])?;

    let staking_record = &mut ctx.accounts.staking_record;
    ***staking_record = StakingRecord::init(user, nft_mint_key, bump, staking_period);
    emit!(StakeEvent {
        staker: user,
        staking_period: staking_period,
        staked_at: current_time,
        nft_mint: nft_mint_key,
        collection_mint: ctx.accounts.stake_details.collection
    });
    Ok(())
}
