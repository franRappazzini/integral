use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Provided token mint is not valid")]
    InvalidTokenMint,
}
