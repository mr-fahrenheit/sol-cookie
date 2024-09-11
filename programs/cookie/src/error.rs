use anchor_lang::prelude::*;

#[error_code]
pub enum CookieError {
    #[msg("Invalid argument")]
    InvalidArgument,
    #[msg("Numerical overflow")]
    NumericalOverflow,
    #[msg("Invalid mint account")]
    InvalidMintAccount,
    #[msg("Insufficient balance")]
    InsufficientBalance,
    #[msg("Insufficient stake")]
    InsufficientStake,
    #[msg("CPI not allowed")]
    CPINotAllowed,
    #[msg("Unauthorized program found")]
    UnauthorizedProgramFound,
    #[msg("Rate limit")]
    RateLimit,
}
