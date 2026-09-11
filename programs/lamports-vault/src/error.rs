use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Custom error message")]
    CustomError,
}

#[error_code]
pub enum VaultError{
    #[msg("Withdrawal amount exceeds the maximum allowed limit")]
    ExceedsMaxWithdraw,
}
