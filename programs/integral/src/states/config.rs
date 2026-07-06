use anchor_lang::prelude::*;

use crate::{error::IntegralError, DISCRIMINATOR};

#[account]
#[derive(InitSpace)]
pub struct Config {
    pub authority: Pubkey,
    pub reward_mint: Pubkey,
    pub reward_amount: u64,
    pub total_claimed: u64,
    pub fee_bps: u16,
    pub winner_settled: bool,
    pub bump_reward_vault: u8,
    pub bump: u8,
}

impl Config {
    pub const SIZE: usize = DISCRIMINATOR + Config::INIT_SPACE;

    pub fn add_rewards(&mut self, amount: u64) -> Result<()> {
        self.reward_amount = self
            .reward_amount
            .checked_add(amount)
            .ok_or(IntegralError::MathOverflow)?;
        Ok(())
    }

    pub fn claim(&mut self, amount: u64) -> Result<()> {
        self.total_claimed = self
            .total_claimed
            .checked_add(amount)
            .ok_or(IntegralError::MathOverflow)?;
        Ok(())
    }
}
