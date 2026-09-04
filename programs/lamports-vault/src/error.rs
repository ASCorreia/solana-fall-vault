use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Custom error message")]
    CustomError,
    #[msg("Withdrawal amount exceeds the maximum allowed limit.")]
    WithdrawalExceedsMaxLimit,
}
