mod instructions;
mod state;
use anchor_lang::prelude::*;
use instructions::init_staking::*;
use instructions::stake::*;
declare_id!("ANcqthEwGEQwKh7JoPLTsbW3atyf1civUBnQ22iTgRH1");

#[program]
pub mod nft_stake {

    use super::*;
    pub fn init(ctx: Context<InitStaking>, reward: u64) -> Result<()> {
        init_staking_handler(ctx, reward)
    }

    pub fn stake(ctx: Context<Stake>, staking_period: u8) -> Result<()> {
        stake_handler(ctx, staking_period)
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
}
