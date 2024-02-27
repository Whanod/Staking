use anchor_lang::prelude::*;
use anchor_spl::token::{set_authority, Mint, SetAuthority, Token};

use crate::state::{events::InitEvent, stake_details::Deatils};

#[derive(Accounts)]
pub struct InitStaking<'info> {
    #[account(init,payer=creator,space=Deatils::LEN,seeds=[b"stake",collection_address.key().as_ref(),creator.key().as_ref()],bump)]
    pub stake_details: Account<'info, Deatils>,
    pub system_program: Program<'info, System>,
    #[account(mut)]
    pub creator: Signer<'info>,
    #[account(mut,mint::authority=creator)]
    pub token_mint: Account<'info, Mint>,
    pub collection_address: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    /// CHECK: This account is not read or written
    #[account(
            seeds = [
                b"token-authority",
                stake_details.key().as_ref()
            ],
            bump
        )]
    pub token_authority: UncheckedAccount<'info>,

    /// CHECK: This account is not read or written
    #[account(
            seeds = [
                b"nft-authority",
                stake_details.key().as_ref()
            ],
            bump
        )]
    pub nft_authority: UncheckedAccount<'info>,
}
impl<'info> InitStaking<'info> {
    pub fn transfer_auth(&self) -> CpiContext<'_, '_, '_, 'info, SetAuthority<'info>> {
        let cpi_accounts = SetAuthority {
            account_or_mint: self.token_mint.to_account_info(),
            current_authority: self.creator.to_account_info(),
        };

        let cpi_program = self.token_program.to_account_info();

        CpiContext::new(cpi_program, cpi_accounts)
    }
}
pub fn init_staking_handler(ctx: Context<InitStaking>, reward: u64) -> Result<()> {
    let reward_mint = ctx.accounts.token_mint.key();
    let collection = ctx.accounts.collection_address.key();
    let creator = ctx.accounts.creator.key();
    let stake_bump = ctx.bumps.stake_details;
    let token_auth_bump = ctx.bumps.token_authority;
    let nft_auth_bump = ctx.bumps.nft_authority;
    let token_authority = ctx.accounts.token_authority.key();
    let current_time = Clock::get()?.unix_timestamp;
    set_authority(
        ctx.accounts.transfer_auth(),
        anchor_spl::token::spl_token::instruction::AuthorityType::MintTokens,
        Some(token_authority),
    )?;
    let stake_details = &mut ctx.accounts.stake_details;

    **stake_details = Deatils::init(
        creator,
        reward_mint,
        reward,
        collection,
        stake_bump,
        token_auth_bump,
        nft_auth_bump,
    );
    emit!(InitEvent {
        creator: creator,
        collection_mint: collection,
        reward_mint: reward_mint,
        init_at: current_time
    });

    Ok(())
}
