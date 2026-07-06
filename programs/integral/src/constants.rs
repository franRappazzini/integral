use anchor_lang::constant;

pub const DISCRIMINATOR: usize = 8;

#[constant]
pub const CONFIG_SEED: &[u8] = b"config";

#[constant]
pub const MARKET_SEED: &[u8] = b"market";

#[constant]
pub const VAULT_SEED: &[u8] = b"vault";

#[constant]
pub const MAX_FEE_BPS: u16 = 500; // 5%
