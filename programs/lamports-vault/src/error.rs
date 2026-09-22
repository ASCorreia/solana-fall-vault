use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Custom error message")]
    CustomError,
    #[msg("You can't withdraw. Your limit has exceeded.")]
    WithdrawError,
}
