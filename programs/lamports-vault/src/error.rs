use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Custom error message")]
    CustomError,
    #[msg("The requested withdrawal amount exceeds the per-transaction limit")]
    ExceedsMaxWithdraw,
}

