use anchor_lang::prelude::*;

use crate::DISCRIMINATOR;

#[account]
#[derive(InitSpace)]
pub struct Config {
    pub authority: Pubkey,
    pub reward_mint: Pubkey,
    pub reward_amount: u64,
    pub fee_bps: u16,
    pub bump_reward_vault: u8,
    pub bump: u8,
}

impl Config {
    pub const SIZE: usize = DISCRIMINATOR + Config::INIT_SPACE;
}
