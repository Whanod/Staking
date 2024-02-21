mod instructions;
mod state;
mod utils;
use anchor_lang::prelude::*;
use instructions::{claim_reward::*, close_staking::*, init_staking::*, stake::*, unstake::*};

declare_id!("ATfdE39GhVCzGEeX8kVnbPwb1Uur7fBX8jCU1SrL3Swq");

#[program]
pub mod staking {

    use super::*;
    pub fn init(ctx: Context<InitStaking>, reward: u64) -> Result<()> {
        init_staking_handler(ctx, reward)
    }

    pub fn stake(ctx: Context<Stake>, staking_period: u8) -> Result<()> {
        stake_handler(ctx, staking_period)
    }

    pub fn claim(ctx: Context<ClaimReward>) -> Result<()> {
        claim_handler(ctx)
    }
    pub fn unstake(ctx: Context<Unstake>) -> Result<()> {
        unstake_handler(ctx)
    }
    pub fn close_staking(ctx: Context<CloseStaking>) -> Result<()> {
        close_staking_handler(ctx)
    }
}
#[error_code]
pub enum StakeError {
    #[msg("Undefined Stake Period")]
    StakePeriodError,
    #[msg("the given mint account doesn't belong to NFT")]
    TokenNotNFT,
    #[msg("the given token account has no NFT")]
    TokenAccountEmpty,
    #[msg("the collection field in the metadata is not verified")]
    CollectionNotVerified,
    #[msg("the collection doesn't match the staking details")]
    InvalidCollection,
    #[msg("the staking is not currently active")]
    StakingInactive,
    #[msg("Unable To Subtract")]
    ProgramSubError,
    #[msg("Unable To Multiply")]
    ProgramMulError,
    #[msg("Failed to convert the time to u64")]
    FailedTimeConversion,
    #[msg("Staking Period is not over yet")]
    UnStakePeriodError,
}
