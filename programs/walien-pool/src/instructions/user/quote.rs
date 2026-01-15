use crate::{constants::CONFIG_SEED, state::GlobalConfig, utils::calculate_swap_from_config};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Quote<'info> {
    #[account(
        seeds = [
            CONFIG_SEED
        ],
        bump
    )]
    pub global_config_account: Account<'info, GlobalConfig>,
}

impl<'info> Quote<'info> {
    pub fn apply(ctx: &Context<Quote>, amount: u64) -> Result<u64> {
        let cfg = &ctx.accounts.global_config_account;
        let calculation_result = calculate_swap_from_config(cfg, amount)?;
        Ok(calculation_result.amount_out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::{USDC_DECIMALS, WALIEN_DECIMALS};
    use crate::{errors::ErrorCode, state::GlobalConfig};

    fn normalize_price_to_ui(sqrt_price_x64: u128) -> f64 {
        let sqrt_f = sqrt_price_x64 as f64 / (1u128 << 64) as f64;
        let raw_price = sqrt_f * sqrt_f;
        let scale = 10f64.powi((WALIEN_DECIMALS as i32 - USDC_DECIMALS as i32) as i32);
        raw_price * scale
    }

    #[test]
    fn test_price_swap() {
        let mut total_sold = 0u64;

        let tick_upper = -61081;
        let sqrtx64 = 18446744073709552;
        let liqudity = 106167919507750;

        let mut cfg = GlobalConfig {
            admin: Pubkey::default(),
            usdc_mint: Pubkey::default(),
            walien_mint: None,
            is_sale_active: true,
            is_claim_active: false,
            bump: 0,
            liqudity: liqudity,
            // liqudity: safe_liquidity_from_b_only(100_000u64 * 10u64.pow(RECEIVE_DECIMALS as u32) as u64, tick_upper)?,
            initial_sqrt_price_x64: sqrtx64,
            // initial_sqrt_price_x64: 583635577551101000,
            tick_upper: tick_upper,
            fee_bps: 0, // for simple testing
            available_for_swap_in_usdc: 0,
            possition_index: 1,
        };
        let amount_in: u64 = 1000 * 1000_000;
        let target_total_input: u64 = 20_000 * 1000_000;
        let mut accumulated_input: u64 = 0;

        while accumulated_input < target_total_input {
            let res = super::calculate_swap_from_config(&cfg, amount_in).unwrap();
            cfg.initial_sqrt_price_x64 = res.next_price;
            accumulated_input += res.amount_in;
            total_sold += res.amount_out;
        }

        let human_price = normalize_price_to_ui(cfg.initial_sqrt_price_x64);
        println!("total sold: {}", total_sold / 1000_000_000);
        println!("total input: {}", accumulated_input / 1000_000);
        println!(
            "final price: {} ({:.12} USDC)",
            cfg.initial_sqrt_price_x64, human_price
        );
    }

    fn config_default() -> Result<GlobalConfig> {
        Ok(GlobalConfig {
            admin: Pubkey::default(),
            usdc_mint: Pubkey::default(),
            walien_mint: None,
            is_sale_active: true,
            is_claim_active: false,
            bump: 0,
            liqudity: 100000 * 10u64.pow(WALIEN_DECIMALS as u32) as u128,
            // liqudity: safe_liquidity_from_b_only(100_000u64 * 10u64.pow(RECEIVE_DECIMALS as u32) as u64, tick_upper)?,
            initial_sqrt_price_x64: 583635577511010402034,
            // initial_sqrt_price_x64: 583635577551101000,
            tick_upper: 0,
            fee_bps: 0, // for simple testing
            available_for_swap_in_usdc: 0,
            possition_index: 1,
        })
    }

    #[test]
    fn test_quote_price_moves_after_swap() {
        let cfg = config_default().unwrap();

        let amount_in = 1000_000u64; // 1 USDC (6 decimals) – example
        let res = super::calculate_swap_from_config(&cfg, amount_in).unwrap();

        assert!(res.amount_out > 0, "amount_out must be > 0");
        assert!(
            res.next_price < cfg.initial_sqrt_price_x64,
            "sqrt_price must DECREASE for zero_for_one=true"
        );
    }

    #[test]
    fn test_price_monotonically_increases_with_input() {
        let mut cfg = config_default().unwrap();

        let res1 = super::calculate_swap_from_config(&cfg, 100_000u64 * 10u64.pow(6)).unwrap();
        cfg.initial_sqrt_price_x64 = res1.next_price;
        let res2 = super::calculate_swap_from_config(&cfg, 100__000u64 * 10u64.pow(6)).unwrap();

        assert!(
            res2.next_price < res1.next_price,
            "Larger input must move price further down when zero_for_one=true"
        );
    }

    #[test]
    fn test_error_on_zero_liquidity() {
        let mut cfg = config_default().unwrap();
        cfg.liqudity = 0;

        let amount_in = 100_000u64;

        let err = super::calculate_swap_from_config(&cfg, amount_in).unwrap_err();
        assert_eq!(err, ErrorCode::LiquidityZero.into());
    }

    #[test]
    fn test_quote_with_fee() {
        let mut cfg_fee = config_default().unwrap();
        cfg_fee.fee_bps = 30; // 0.30%

        let cfg_no_fee = config_default().unwrap();

        let amount_in = 1_000_000_000u64; // larger so amount_out is visible even after rounding

        let out_fee = super::calculate_swap_from_config(&cfg_fee, amount_in)
            .unwrap()
            .amount_out;
        let out_no_fee = super::calculate_swap_from_config(&cfg_no_fee, amount_in)
            .unwrap()
            .amount_out;

        assert!(
            out_fee < out_no_fee,
            "With fee applied amount_out must be smaller"
        );
    }

    #[test]
    fn test_small_amount_gives_small_price_movement() {
        let cfg = config_default().unwrap();

        let res = super::calculate_swap_from_config(&cfg, 1).unwrap(); // 1 wei of USDC

        assert!(
            res.next_price < cfg.initial_sqrt_price_x64,
            "Even smallest input should move the price down when zero_for_one=true"
        );
        assert!(
            cfg.initial_sqrt_price_x64 - res.next_price < 1_000_000_000,
            "Price delta for tiny input must be small"
        );
    }

    #[test]
    fn test_zero_input_error() {
        let cfg = config_default().unwrap();
        let err = super::calculate_swap_from_config(&cfg, 0).unwrap_err();
        assert_eq!(err, ErrorCode::ZeroTradableAmount.into());
    }

    #[test]
    fn test_price_progression_across_inputs() {
        let mut cfg = config_default().unwrap();
        let inputs = [
            100 * 10u64.pow(6),
            1_000 * 10u64.pow(6),
            10_000 * 10u64.pow(6),
            80_000 * 10u64.pow(6),
        ];

        let prices: Vec<u128> = inputs
            .iter()
            .map(|amount| {
                println!("price: {}", cfg.initial_sqrt_price_x64);
                println!("AMOUNT: {}", amount);
                let res = super::calculate_swap_from_config(&cfg, *amount).unwrap();
                cfg.initial_sqrt_price_x64 = res.next_price;
                println!(
                    "Spent: {} AMOUNT OUT raw (receive units): {}",
                    amount / 10u64.pow(6),
                    res.amount_out / 10u64.pow(9)
                );

                println!("LIQ (raw units): {}", cfg.liqudity);

                res.next_price
            })
            .collect();

        for (idx, price) in prices.iter().enumerate() {
            let real_price = normalize_price_to_ui(*price);
            println!(
                "input[{}] => sqrt_price_x64 = {:>20}, price = {:.12}",
                idx, price, real_price
            );
        }

        for window in prices.windows(2) {
            assert!(
                window[1] > window[0],
                "Price should increase as input grows (zero_for_one=true)"
            );
        }
    }
}
