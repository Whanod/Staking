use crate::instructions::send_pnft::send_pnft;
use crate::state::{events::StakeEvent, stake_details::Deatils, staking_record::StakingRecord};
use crate::StakeError;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::metadata::{MasterEditionAccount, TokenRecordAccount};
use anchor_spl::metadata::{Metadata, MetadataAccount};
use anchor_spl::token::{Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(seeds=[b"stake",stake_details.collection.as_ref(),stake_details.creator.as_ref()],bump=stake_details.stake_bump)]
    pub stake_details: Box<Account<'info, Deatils>>,
    #[account(init,payer=staker,space=StakingRecord::LEN+8,seeds=[b"staking-record",stake_details.key().as_ref(),nft_mint.key().as_ref()],bump)]
    pub staking_record: Box<Account<'info, StakingRecord>>,
    pub system_program: Program<'info, System>,
    #[account(mut)]
    /// CHECK: Sysvar
    pub staker: UncheckedAccount<'info>,
    #[account(mint::decimals = 0,constraint= nft_mint.supply == 1 @ StakeError::TokenNotNFT )]
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
    #[account(mut,
        seeds = [
            b"metadata",Metadata::id().as_ref(),nft_mint.key().as_ref(),b"token_record",nft_token.key().as_ref()
        ],seeds::program=Metadata::id(),bump
    )]
    /// CHECK: Sysvar
    pub token_record: UncheckedAccount<'info>,
    #[account(mut,
        seeds = [
            b"metadata",Metadata::id().as_ref(),nft_mint.key().as_ref(),b"token_record",nft_custody.key().as_ref()
        ],seeds::program=Metadata::id(),bump
    )]
    /// CHECK: Sysvar
    pub token_record_dest: UncheckedAccount<'info>,
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
    // let id = Metadata::id();
    // let auth_seeds = &[
    //     &b"metadata"[..],
    //     &id.as_ref(),
    //     &nft_mint_key.as_ref(),
    //     &b"edition"[..],
    //     &[ctx.bumps.nft_edition],
    // ];

    let bump = ctx.bumps.staking_record;
    send_pnft(
        &ctx.accounts.metadata_program,
        &ctx.accounts.staker,
        &ctx.accounts.staker,
        &ctx.accounts.staker,
        &ctx.accounts.nft_token,
        &ctx.accounts.nft_custody,
        &ctx.accounts.nft_authority,
        &ctx.accounts.nft_mint,
        &ctx.accounts.nft_metadata,
        &ctx.accounts.nft_edition,
        &ctx.accounts.system_program,
        &ctx.accounts.token_program,
        &ctx.accounts.associated_token_program,
        &ctx.accounts.sysvar_instructions,
        &ctx.accounts.token_record,
        &ctx.accounts.token_record_dest,
        &ctx.accounts.auth_program,
        &ctx.accounts.auth_rules,
        false,
        &ctx.accounts.stake_details,
    )?;

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
