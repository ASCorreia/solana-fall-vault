use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Custom error message")]
    CustomError,
    #[msg("Cannot withdraw as exceeds max withdraw amount")]
    ExceedsMaxWithdraw,
}
