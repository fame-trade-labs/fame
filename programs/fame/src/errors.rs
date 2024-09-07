use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid token name")]
    InvalidTokenName,
    #[msg("Invalid token")]
    InvalidToken,
    #[msg("Invalid token symbol")]
    InvalidTokenSymbol,
    #[msg("Invalid social account URL")]
    InvalidSocialAccountUrl,
    #[msg("Insufficient balance for burning")]
    InsufficientBalance,
    #[msg("Liquidity pool balance too low")]
    InsufficientLiquidity,
    #[msg("Invalid mint amount")]
    InvalidMintAmount,
    #[msg("Invalid burn amount")]
    InvalidBurnAmount,
    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,
    #[msg("Unauthorized access")]
    Unauthorized,
    #[msg("Arithmetic underflow occurred")]
    ArithmeticUnderflow,
    #[msg("Invalid bonding curve parameters")]
    InvalidBondingCurveParameters,
}
