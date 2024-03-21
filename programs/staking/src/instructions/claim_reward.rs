use crate::{
    state::{events::ClaimEvent, stake_details::Deatils, staking_record::StakingRecord},
    utils::calc_reward::calc_reward,
    StakeError,
};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{mint_to, Mint, MintTo, Token, TokenAccount},
};

#[derive(Accounts)]
pub struct ClaimReward<'info> {
    #[account(seeds= [b"stake", 
    stake_details.collection.as_ref(),
    stake_details.creator.as_ref()
],bump=stake_details.stake_bump,has_one=reward_mint)]
    pub stake_details: Account<'info, Deatils>,
    #[account(mut)]
    pub staker: Signer<'info>,
    #[account(mut,seeds = [
        b"staking-record", 
        stake_details.key().as_ref(),
        staking_record.nft_mint.as_ref(),
    ],
    bump = staking_record.bump,
    has_one = staker)]
    pub staking_record: Account<'info, StakingRecord>,
    /// CHECK: This account is not read or written
    #[account(
            seeds = [
                b"token-authority",
                stake_details.key().as_ref(),
            ],
            bump
        )]
    pub token_authority: UncheckedAccount<'info>,
    #[account(
        mut,
        mint::authority = token_authority,
    )]
    pub reward_mint: Account<'info, Mint>,
    #[account(init_if_needed,
        payer = staker,associated_token::mint = reward_mint,
        associated_token::authority = staker)]
    pub reward_receive_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
impl<'info> ClaimReward<'info> {
    pub fn mint_token_ctx(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        let cpi_accounts = MintTo {
            mint: self.reward_mint.to_account_info(),
            to: self.reward_receive_account.to_account_info(),
            authority: self.token_authority.to_account_info(),
        };
        let cpi_program = self.token_program.to_account_info();
        CpiContext::new(cpi_program, cpi_accounts)
    }
}
pub fn claim_handler(ctx: Context<ClaimReward>) -> Result<()> {
    const SECONDS_IN_A_MONTH: i64 = 2592000;
    let init_at: i64 = ctx.accounts.stake_details.init_at;

    let staking_status = ctx.accounts.stake_details.is_active;
    require_eq!(staking_status, true, StakeError::StakingInactive);

    let claimed_last = ctx.accounts.staking_record.last_claimed;
    let staking_period = ctx.accounts.staking_record.staking_period;
    let base_reward = ctx.accounts.stake_details.reward;
    let (reward_tokens, current_time) =
        calc_reward(claimed_last, staking_period, base_reward).unwrap();
    require_gt!(
        current_time,
        init_at + SECONDS_IN_A_MONTH,
        StakeError::ClaimError
    );
    let token_auth_bump = ctx.accounts.stake_details.token_auth_bump;
    let stake_details_key = ctx.accounts.stake_details.key();
    let authority_seed = &[
        &b"token-authority"[..],
        &stake_details_key.as_ref(),
        &[token_auth_bump],
    ];
    mint_to(
        ctx.accounts
            .mint_token_ctx()
            .with_signer(&[&authority_seed[..]]),
        reward_tokens,
    )?;
    ctx.accounts.staking_record.last_claimed = current_time;
    emit!(ClaimEvent {
        reward: reward_tokens,
        reward_mint: ctx.accounts.reward_mint.key(),
        collection_mint: ctx.accounts.stake_details.collection.key(),
        nft_mint: ctx.accounts.staking_record.nft_mint.key(),
        staker: ctx.accounts.staking_record.staker.key(),
        claimed_at: current_time
    });
    Ok(())
}
