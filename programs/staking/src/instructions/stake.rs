use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::metadata::{Metadata, MetadataAccount};

use anchor_spl::metadata::MasterEditionAccount;
use anchor_spl::token::{transfer, Mint, Token, TokenAccount, Transfer};

use crate::state::{stake_details::Deatils, staking_record::StakingRecord};
use crate::StakeError;

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(seeds=[b"stake",stake_details.collection.as_ref(),stake_details.creator.as_ref()],bump=stake_details.stake_bump)]
    pub stake_details: Account<'info, Deatils>,
    #[account(init,payer=user,space=StakingRecord::LEN,seeds=[b"staking_record",stake_details.key().as_ref(),nft_mint.key().as_ref()],bump)]
    pub staking_record: Account<'info, StakingRecord>,
    pub system_program: Program<'info, System>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mint::decimals = 0,constraint= nft_mint.supply == 1 @ StakeError::TokenNotNFT )]
    pub nft_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    #[account(mut,associated_token::mint= nft_mint, associated_token::authority = user,constraint= nft_token.amount ==1 @ StakeError::TokenAccountEmpty)]
    pub nft_token: Account<'info, TokenAccount>,
    #[account(
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
        payer = user,
        associated_token::mint = nft_mint,
        associated_token::authority = nft_authority
    )]
    pub nft_custody: Account<'info, TokenAccount>,
}
impl<'info> Stake<'info> {
    pub fn transfer_nft_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.nft_token.to_account_info(),
            to: self.nft_authority.to_account_info(),
            authority: self.user.to_account_info(),
        };
        let cpi_program = self.token_program.to_account_info();
        CpiContext::new(cpi_program, cpi_accounts)
    }
}
pub fn stake_handler(ctx: Context<Stake>, staking_period: u8) -> Result<()> {
    let periods: Vec<u8> = vec![30, 60, 90];
    require_eq!(
        periods.contains(&staking_period),
        true,
        StakeError::StakePeriodError
    );

    let staking_status = ctx.accounts.stake_details.is_active;
    require_eq!(staking_status, true, StakeError::StakingInactive);
    let user = ctx.accounts.user.key();
    let nft_mint = ctx.accounts.nft_mint.key();
    let bump = ctx.bumps.staking_record;
    transfer(ctx.accounts.transfer_nft_ctx(), 1)?;
    let staking_record = &mut ctx.accounts.staking_record;
    **staking_record = StakingRecord::init(user, nft_mint, bump, staking_period);
    Ok(())
}
