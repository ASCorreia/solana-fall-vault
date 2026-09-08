use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Custom error message")]
    CustomError,

#[msg("STOP! You have taketh out too much money.")]	// custom error message
    ExceedsMaxWithdraw,

}
