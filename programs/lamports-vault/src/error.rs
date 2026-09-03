use anchor_lang::prelude::*;

#[error_code]
pub enum VaultError {
    #[msg("Custom error message")]
    CustomError,
    #[msg("Withdraw amount exceeds maximum allowed")]
    ExceedsMaxWithdraw,
}
