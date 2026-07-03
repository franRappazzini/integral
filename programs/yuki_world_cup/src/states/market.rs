use anchor_lang::prelude::*;

use crate::DISCRIMINATOR;

#[account]
#[derive(InitSpace)]
pub struct Market {
    pub mint: Pubkey,
    pub receipt_mint: Pubkey,
    pub total_deposited: u64,
    pub fee_bps: u16,
    pub is_open: bool,
    pub bump_vault: u8,
    pub bump: u8,
}

impl Market {
    pub const SIZE: usize = DISCRIMINATOR + Market::INIT_SPACE;
}
