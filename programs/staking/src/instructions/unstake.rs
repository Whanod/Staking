use crate::{
    instructions::send_pnft::send_pnft,
    state::{events::UnstakeEvent, stake_details::*, staking_record::StakingRecord},
    utils::calc_reward::calc_reward,
    StakeError,
};
use anchor_spl::metadata::{MasterEditionAccount, Metadata, MetadataAccount};

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{
        close_account, mint_to, transfer, CloseAccount, Mint, MintTo, Token, TokenAccount, Transfer,
    },
};

#[derive(Accounts)]
pub struct Unstake<'info> {
    pub metadata_program: Program<'info, Metadata>,
    /// CHECK: Sysvar
    pub sysvar_instructions: AccountInfo<'info>,
    #[account(seeds=[b"stake",stake_details.collection.as_ref(),stake_details.creator.as_ref()],bump=stake_details.stake_bump,has_one=reward_mint)]
    pub stake_details: Box<Account<'info, Deatils>>,
    #[account(mut, signer)]
    /// CHECK: Sysvar
    pub staker: UncheckedAccount<'info>,
    #[account(mut,seeds=[b"staking-record",stake_details.key().as_ref(),staking_record.nft_mint.as_ref()],bump=staking_record.bump,has_one=nft_mint,has_one=staker,close=staker)]
    pub staking_record: Box<Account<'info, StakingRecord>>,
    #[account(mut,mint::authority= token_authority)]
    pub reward_mint: Box<Account<'info, Mint>>,
    #[account(init_if_needed,payer=staker,associated_token::mint=reward_mint,associated_token::authority=staker)]
    pub reward_receive_account: Box<Account<'info, TokenAccount>>,
    #[account(mut,mint::decimals=0,constraint = nft_mint.supply == 1 @ StakeError::TokenNotNFT,)]
    pub nft_mint: Box<Account<'info, Mint>>,
    #[account(
        init_if_needed,
        payer = staker,
        associated_token::mint = nft_mint,
        associated_token::authority = staker,
    )]
    pub nft_receive_account: Box<Account<'info, TokenAccount>>,
    #[account(
        seeds = [
            b"token-authority",
            stake_details.key().as_ref(),
        ],
        bump = stake_details.token_auth_bump
    )]
    /// CHECK: This account is not read or written
    pub token_authority: UncheckedAccount<'info>,

    /// CHECK: This account is not read or written
    #[account(mut,
        seeds = [
            b"nft-authority",
            stake_details.key().as_ref()
        ],
        bump = stake_details.nft_auth_bump
    )]
    pub nft_authority: UncheckedAccount<'info>,
    #[account(mut,associated_token::mint=nft_mint,associated_token::authority=nft_authority,constraint = nft_custody.amount == 1 @ StakeError::TokenAccountEmpty,)]
    pub nft_custody: Box<Account<'info, TokenAccount>>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
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
    /// CHECK: Sysvar
    pub auth_program: AccountInfo<'info>,
    /// CHECK: Sysvar
    pub auth_rules: AccountInfo<'info>,
    #[account(mut,
            seeds = [
                b"metadata",Metadata::id().as_ref(),nft_mint.key().as_ref(),b"token_record",nft_custody.key().as_ref()
            ],seeds::program=Metadata::id(),bump
        )]
    /// CHECK: Sysvar
    pub token_record: UncheckedAccount<'info>,
    #[account(mut,
            seeds = [
                b"metadata",Metadata::id().as_ref(),nft_mint.key().as_ref(),b"token_record",nft_receive_account.key().as_ref()
            ],seeds::program=Metadata::id(),bump
        )]
    /// CHECK: Sysvar
    pub token_record_dest: UncheckedAccount<'info>,
}
impl<'info> Unstake<'info> {
    pub fn mint_reward_ctx(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        let cpi_accounts = MintTo {
            mint: self.reward_mint.to_account_info(),
            to: self.reward_receive_account.to_account_info(),
            authority: self.token_authority.to_account_info(),
        };
        let cpi_program = self.token_program.to_account_info();
        CpiContext::new(cpi_program, cpi_accounts)
    }

    pub fn close_account_ctx(&self) -> CpiContext<'_, '_, '_, 'info, CloseAccount<'info>> {
        let cpi_accounts = CloseAccount {
            account: self.nft_custody.to_account_info(),
            destination: self.staker.to_account_info(),
            authority: self.nft_authority.to_account_info(),
        };
        let cpi_program = self.token_program.to_account_info();
        CpiContext::new(cpi_program, cpi_accounts)
    }
}
pub fn unstake_handler(ctx: Context<Unstake>) -> Result<()> {
    //change to 2592000 in production
    const SECONDS_IN_A_MONTH: i64 = 2592000;
    let stake_details = &ctx.accounts.stake_details;
    let staked_at = ctx.accounts.staking_record.staked_at;
    let staking_active = stake_details.is_active;
    let token_auth_bump = stake_details.token_auth_bump;
    let nft_auth_bump = stake_details.nft_auth_bump;
    let staking_period = ctx.accounts.staking_record.staking_period;
    let last_claimed = ctx.accounts.staking_record.last_claimed;
    let clock = Clock::get().unwrap();
    let current_time = clock.unix_timestamp;
    let staking_period_i64 = staking_period as i64;
    let stake_details_key = stake_details.key();

    // require_gt!(
    //     current_time,
    //     staked_at + (SECONDS_IN_A_MONTH * staking_period_i64),
    //     StakeError::UnStakePeriodError
    // );
    let base_reward = ctx.accounts.stake_details.reward;
    let token_auth_seed = &[
        &b"token-authority"[..],
        &stake_details_key.as_ref(),
        &[token_auth_bump],
    ];
    let nft_auth_seed = &[
        &b"nft-authority"[..],
        &stake_details_key.as_ref(),
        &[nft_auth_bump],
    ];
    let (reward_tokens, _) = calc_reward(last_claimed, staking_period, base_reward).unwrap();

    if staking_active {
        mint_to(
            ctx.accounts
                .mint_reward_ctx()
                .with_signer(&[&token_auth_seed[..]]),
            reward_tokens,
        )?;
    }
    send_pnft(
        &ctx.accounts.metadata_program,
        &ctx.accounts.nft_authority,
        &ctx.accounts.nft_authority,
        &ctx.accounts.staker,
        &ctx.accounts.nft_custody,
        &ctx.accounts.nft_receive_account,
        &ctx.accounts.staker,
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
        true,
        &ctx.accounts.stake_details,
    )?;
    close_account(
        ctx.accounts
            .close_account_ctx()
            .with_signer(&[&nft_auth_seed[..]]),
    )?;
    emit!(UnstakeEvent {
        staker: ctx.accounts.staker.key(),
        nft_mint: ctx.accounts.nft_mint.key(),
        collection_mint: ctx.accounts.stake_details.collection.key(),
        staked_at: staked_at,
        unstaked_at: current_time,
    });

    Ok(())
}
