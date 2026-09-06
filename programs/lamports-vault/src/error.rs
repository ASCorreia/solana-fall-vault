use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Custom error message")]
    CustomError,
    #[msg("Not letting you take more than allowed")]
    WithdrawLimitExceeded,
}
