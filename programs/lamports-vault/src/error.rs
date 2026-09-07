use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Custom error message")]
    CustomError,
    #[msg("Amount is over the per-transaction withdraw limit")]
    MaxWithdrawExceeded,
}
