use crate::{
    errors::ErrorCode,
    orca_math::{compute_swap, sqrt_price_from_tick_index, SwapStepComputation},
    state::GlobalConfig,
};
use anchor_lang::prelude::*;

pub fn calculate_swap_from_config(cfg: &GlobalConfig, amount: u64) -> Result<SwapStepComputation> {
    if amount == 0 {
        return Err(ErrorCode::ZeroTradableAmount.into());
    }

    if cfg.liqudity == 0 {
        return Err(ErrorCode::LiquidityZero.into());
    }
    let fee_rate = cfg.fee_bps as u32;
    let amount_specified_is_input = true;
    let a_to_b = false;

    Ok(compute_swap(
        amount,
        fee_rate,
        cfg.liqudity as u128,
        cfg.initial_sqrt_price_x64,
        sqrt_price_from_tick_index(cfg.tick_upper),
        amount_specified_is_input,
        a_to_b,
    )?)
}
