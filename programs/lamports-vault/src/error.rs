use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Custom error message")]
    CustomError,
    #[msg("Withdraw amount exceeds the vault max_withdraw limit")]
    ExceedsMaxWithdraw,
}
