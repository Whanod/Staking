use crate::StakeError;
use anchor_lang::prelude::*;

pub fn calc_reward(last_claimed: i64, lock_period: u8, base_reward: u64) -> Result<(u64, i64)> {
    let clock = Clock::get().unwrap();
    let current_time = clock.unix_timestamp;
    let rewardable_time = current_time
        .checked_sub(last_claimed)
        .ok_or(StakeError::ProgramSubError)?;
    let current_reward_rate = base_reward
        .checked_mul(lock_period.into())
        .ok_or(StakeError::ProgramMulError)?;
    let rewardable_time_u64 = match u64::try_from(rewardable_time) {
        Ok(time) => time,
        _ => return err!(StakeError::FailedTimeConversion),
    };
    let token_rewards = current_reward_rate
        .checked_mul(rewardable_time_u64)
        .ok_or(StakeError::ProgramMulError)?;
    Ok((token_rewards, current_time))
}
