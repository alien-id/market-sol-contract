use anchor_lang::prelude::*;
use std::num::TryFromIntError;

#[error_code]
#[derive(PartialEq)]
pub enum ErrorCode {
    // ----- Tick math / price math -----
    #[msg("Invalid start tick index provided.")]
    InvalidStartTick, // 6001
    #[msg("Tick-spacing is not supported")]
    InvalidTickSpacing, // 6004
    #[msg("Tick not found within tick array")]
    TickNotFound, // 6009
    #[msg("Provided tick index is either out of bounds or uninitializable")]
    InvalidTickIndex, // 6010
    #[msg("Provided sqrt price out of bounds")]
    SqrtPriceOutOfBounds, // 6011

    // ----- Liquidity math -----
    #[msg("Liquidity amount must be greater than zero")]
    LiquidityZero, // 6012
    #[msg("Liquidity amount must be less than i64::MAX")]
    LiquidityTooHigh, // 6013
    #[msg("Liquidity overflow")]
    LiquidityOverflow, // 6014
    #[msg("Liquidity underflow")]
    LiquidityUnderflow, // 6015
    #[msg("Tick liquidity net underflowed or overflowed")]
    LiquidityNetError, // 6016

    // ----- Numeric math -----
    #[msg("Unable to divide by zero")]
    DivideByZero, // 6006
    #[msg("Unable to cast number into BigInt")]
    NumberCastError, // 6007
    #[msg("Unable to down cast number")]
    NumberDownCastError, // 6008
    #[msg("Did not meet token min")]
    TokenMinSubceeded, // 0x1782 (6018)
    #[msg("Multiplication with shift right overflow")]
    MultiplicationShiftRightOverflow, // 6030
    #[msg("Muldiv overflow")]
    MulDivOverflow, // 6031
    #[msg("Invalid div_u256 input")]
    MulDivInvalidInput, // 6032
    #[msg("Multiplication overflow")]
    MultiplicationOverflow, // 6033

    // ----- Swap math (amount deltas, thresholds) -----
    #[msg("There are no tradable amount to swap.")]
    ZeroTradableAmount, // 6035
    #[msg("Amount out below minimum threshold")]
    AmountOutBelowMinimum, // 6036
    #[msg("Amount in above maximum threshold")]
    AmountInAboveMaximum, // 6037
    #[msg("Amount calculated overflows")]
    AmountCalcOverflow, // 6039
    #[msg("Amount remaining overflows")]
    AmountRemainingOverflow, // 6040

    // ----- Price limit -----
    #[msg("Provided SqrtPriceLimit not in the same direction as the swap.")]
    InvalidSqrtPriceLimitDirection, // 6034

    #[msg("Exceeded token max")]
    TokenMaxExceeded, // 0x1781 (6017)
    #[msg("Slippage exceeded")]
    SlippageExceeded,
    #[msg("Sale not active")]
    SaleNotActive,
    #[msg("Insufficient available USDC for swap")]
    InsufficientAvailableForSwap,
    #[msg("USDC cap exceeded")]
    UsdcCapExceeded,
    #[msg("User summary underflow")]
    UserSummaryUnderflow,
    #[msg("Nothing to claim")]
    NothingToClaim,
    #[msg("Claim is not active")]
    ClaimIsNotActive,
    #[msg("Invalid Walien token account")]
    InvalidWalienTokenAccount,
    #[msg("Walien is not set")]
    WalienIsNotSet,
    #[msg("Withdrawal not allowed")]
    WithdrawNotAllowed,

    #[msg("Invalid USDC mint")]
    InvalidUsdcMint,
    #[msg("Invalid USDC decimals")]
    InvalidUsdcDecimals,
    #[msg("Invalid Walien decimals")]
    InvalidWalienDecimals,
}

impl From<TryFromIntError> for ErrorCode {
    fn from(_: TryFromIntError) -> Self {
        ErrorCode::NumberCastError
    }
}
