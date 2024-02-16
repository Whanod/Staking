use anchor_lang::prelude::*;

#[account]
pub struct Deatils {
    pub is_active: bool,
    pub creator: Pubkey,
    pub reward_mint: Pubkey,
    pub reward: u64,
    pub collection: Pubkey,
    pub stake_bump: u8,
    pub token_auth_bump: u8,
    pub nft_auth_bump: u8,
}
impl Deatils {
    pub const LEN: usize = 120;
    pub fn init(
        creator: Pubkey,
        reward_mint: Pubkey,
        reward: u64,
        collection: Pubkey,
        stake_bump: u8,
        token_auth_bump: u8,
        nft_auth_bump: u8,
    ) -> Self {
        Self {
            is_active: true,
            creator,
            reward_mint,
            reward,
            collection,
            stake_bump,
            token_auth_bump,
            nft_auth_bump,
        }
    }
    pub fn close_staking(&mut self) -> Result<()> {
        self.is_active = false;
        Ok(())
    }
}
