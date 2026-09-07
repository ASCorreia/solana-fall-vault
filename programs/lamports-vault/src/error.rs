use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Custom error message")]
    CustomError,
    #[msg("Requested amount exceeds the vault's maximum withdrawal limit")]
    ExceedsMaxWithdraw,
}
