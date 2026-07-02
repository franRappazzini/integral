use anchor_lang::prelude::*;

use crate::DISCRIMINATOR;

#[account]
#[derive(InitSpace)]
pub struct Config {
    pub authority: Pubkey,
    pub bump: u8,
}

impl Config {
    pub const SIZE: usize = DISCRIMINATOR + Config::INIT_SPACE;
}
