use anchor_lang::prelude::*;

use crate::{error::ErrorCode, DISCRIMINATOR};

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

    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        self.total_deposited = self
            .total_deposited
            .checked_add(amount)
            .ok_or(ErrorCode::MathOverflow)?;
        Ok(())
    }
}
