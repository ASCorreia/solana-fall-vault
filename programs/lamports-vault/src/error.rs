use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Custom error message")]
    CustomError,
}

#[error_code]
pub enum VaultError {
    #[msg("Insufficient funds in the vault to withdraw requested amount.")]
    InsufficientFunds,

    #[msg("The requested withdrawal amount must be greater than 0.")]
    InvalidWithdrawalAmount,

    #[msg("The requested amount must be equal or less than the maximum allowed limit.")]
    ExceedsMaxWithdrawalCap,
}
