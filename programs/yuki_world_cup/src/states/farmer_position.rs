use anchor_lang::prelude::*;

use crate::{error::ErrorCode, DISCRIMINATOR};

#[account]
#[derive(InitSpace)]
pub struct FarmerPosition {
    pub amount: u64,
    pub is_initialized: bool,
    pub bump: u8,
}

impl FarmerPosition {
    pub const SIZE: usize = DISCRIMINATOR + FarmerPosition::INIT_SPACE;

    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        self.amount = self
            .amount
            .checked_add(amount)
            .ok_or(ErrorCode::MathOverflow)?;
        Ok(())
    }

    pub fn withdraw(&mut self, amount: u64) -> Result<()> {
        self.amount = self
            .amount
            .checked_sub(amount)
            .ok_or(ErrorCode::MathOverflow)?;
        Ok(())
    }
}
