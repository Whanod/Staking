use anchor_lang::prelude::*;

#[event]
pub struct InitEvent {
    pub creator: Pubkey,
    pub collection_mint: Pubkey,
    pub reward_mint: Pubkey,
    pub init_at: i64,
}

#[event]
pub struct StakeEvent {
    pub staker: Pubkey,
    pub staking_period: u8,
    pub staked_at: i64,
    pub nft_mint: Pubkey,
    pub collection_mint: Pubkey,
}

#[event]
pub struct ClaimEvent {
    pub reward: u64,
    pub reward_mint: Pubkey,
    pub collection_mint: Pubkey,
    pub nft_mint: Pubkey,
    pub staker: Pubkey,
    pub claimed_at: i64,
}
#[event]
pub struct UnstakeEvent {
    pub staker: Pubkey,
    pub staked_at: i64,
    pub nft_mint: Pubkey,
    pub collection_mint: Pubkey,
    pub unstaked_at: i64,
}
#[event]
pub struct CloseEvent {
    pub creator: Pubkey,
    pub reward_mint: Pubkey,
    pub closed_at: i64,
    pub collection_mint: Pubkey,
}
