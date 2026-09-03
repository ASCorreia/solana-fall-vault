use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Custom error message")]
    CustomError,
    #[msg("You've exceeded your withdrawal limit")]
    ExceedsMaxWithdraw,
}
