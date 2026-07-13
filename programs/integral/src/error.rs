use anchor_lang::prelude::*;

#[error_code]
pub enum IntegralError {
    #[msg("Provided token mint is not valid")]
    InvalidTokenMint,
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("The provided amount is not valid")]
    InvalidAmount,
    #[msg("Error retrieving or deserializing the metadata extension")]
    NoDeserializeExtension,
    #[msg("The market is not open")]
    MarketIsNotOpen,
    #[msg("The market is not winner. Try withdrawing instead of claiming rewards")]
    MarketIsNotWinner,
    #[msg("The market is winner. Try claiming rewards instead of withdrawing")]
    MarketIsWinner,
}
