use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Custom error message")]
    CustomError,
    #[msg("Withdrawal amount exceeded the maximum allowed limit.")]
    ExceedsMaxWithdraw,
    #[msg("Insufficient sol in the vault.")]
    InsufficientFunds,
}
