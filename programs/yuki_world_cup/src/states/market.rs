use anchor_lang::prelude::*;

use crate::DISCRIMINATOR;

#[account]
#[derive(InitSpace)]
pub struct Market {
    pub mint: Pubkey,
    pub reward_mint: Pubkey,
    pub reward_amount: u64,
    pub bump_vault: u8,
    pub bump_reward_vault: u8,
    pub bump: u8,
}

impl Market {
    pub const SIZE: usize = DISCRIMINATOR + Market::INIT_SPACE;
}
