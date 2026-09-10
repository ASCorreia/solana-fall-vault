use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Withdraw amount exceeds max_withdraw")]
    ExceedsMaxWithdraw,
}