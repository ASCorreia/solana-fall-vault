use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Damn Shits cooked G")]
    ExceedsMaxWithdraw,
}