use anchor_lang::prelude::*;

use crate::DISCRIMINATOR;

#[account]
#[derive(InitSpace)]
pub struct FarmerPosition {
    pub amount: u64,
    pub is_initialized: bool,
    pub bump: u8,
}

impl FarmerPosition {
    pub const SIZE: usize = DISCRIMINATOR + FarmerPosition::INIT_SPACE;
}
