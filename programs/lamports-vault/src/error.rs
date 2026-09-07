use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Custom error message")]
    CustomError,
    #[msg("Maximum withdrawal limit exceeded")] ExceedsMaxWithdraw,
}
