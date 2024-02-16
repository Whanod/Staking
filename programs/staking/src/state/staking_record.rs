use anchor_lang::prelude::*;

#[account]
pub struct StakingRecord {
    pub staker: Pubkey,
    pub nft_mint: Pubkey,
    pub staked_at: i64,
    pub bump: u8,
    pub staking_period: u8,
    pub last_claimed: i64,
}

impl StakingRecord {
    pub const LEN: usize = 32 + 32 + 8 + 8 + 1 + 1;
    pub fn init(staker: Pubkey, nft_mint: Pubkey, bump: u8, staking_period: u8) -> Self {
        let clock = Clock::get().unwrap();
        Self {
            staker,
            nft_mint,
            staked_at: clock.unix_timestamp,
            bump,
            staking_period,
            last_claimed: clock.unix_timestamp,
        }
    }
}
