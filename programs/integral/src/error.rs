use anchor_lang::prelude::*;

#[error_code]
pub enum IntegralError {
    #[msg("Provided token mint is not valid")]
    InvalidTokenMint,
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("The provided amount is not valid")]
    InvalidAmount,
}
