use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("The amount to withdraw exceeds the maximum allowed.")]
    ExceedsMaxWithdraw,
}
