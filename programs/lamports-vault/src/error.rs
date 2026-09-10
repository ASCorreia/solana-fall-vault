use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Custom error message")]
    CustomError,
    #[msg("The amount exceeded the vault's per-transaction limit.")]
    ExceedsMaxWithdraw,
}
