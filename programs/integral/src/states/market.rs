use anchor_lang::prelude::*;

use crate::{error::IntegralError, DISCRIMINATOR};

#[account]
#[derive(InitSpace)]
pub struct Market {
    pub mint: Pubkey,
    pub receipt_mint: Pubkey,
    pub total_deposited: u64,
    pub total_claimed: u64,
    pub collected_fees: u64,
    pub fee_bps: u16,
    pub fees_claimed: bool,
    pub status: MarketStatus,
    pub bump_vault: u8,
    pub bump: u8,
}

impl Market {
    pub const SIZE: usize = DISCRIMINATOR + Market::INIT_SPACE;

    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        self.total_deposited = self
            .total_deposited
            .checked_add(amount)
            .ok_or(IntegralError::MathOverflow)?;
        Ok(())
    }

    pub fn withdraw(&mut self, amount: u64) -> Result<()> {
        self.total_deposited = self
            .total_deposited
            .checked_sub(amount)
            .ok_or(IntegralError::MathOverflow)?;
        Ok(())
    }

    pub fn add_fees(&mut self, amount: u64) -> Result<()> {
        self.collected_fees = self
            .collected_fees
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

    pub fn is_open(&self) -> bool {
        self.status == MarketStatus::Open
    }

    pub fn is_winner(&self) -> bool {
        self.status == MarketStatus::Winner
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, InitSpace, Clone, PartialEq)]
pub enum MarketStatus {
    Open,
    Loser,
    Winner,
}
