use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Maximum withdrawal exceeded. Please try with a lower amount")]
    ExceedsMaxWithdraw,
}
