use anchor_lang::prelude::*;

use crate::DISCRIMINATOR;

#[account]
#[derive(InitSpace)]
pub struct Market {
    pub mint: Pubkey,
    pub fee_bps: u16,
    pub bump_vault: u8,
    pub bump: u8,
}

impl Market {
    pub const SIZE: usize = DISCRIMINATOR + Market::INIT_SPACE;
}
