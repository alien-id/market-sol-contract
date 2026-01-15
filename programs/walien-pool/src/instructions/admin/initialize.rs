use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

use crate::{
    constants::{CONFIG_SEED, USDC_DECIMALS, VAULT_USDC_SEED},
    errors::ErrorCode,
    orca_math::{tick_index_from_sqrt_price, MAX_SQRT_PRICE_X64, MIN_SQRT_PRICE_X64},
    state::GlobalConfig,
};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        space = GlobalConfig::SIZE,
        payer = admin,
        seeds = [
            CONFIG_SEED
        ],
        bump
    )]
    pub global_config_account: Account<'info, GlobalConfig>,
    #[account()]
    pub usdc_mint: Account<'info, Mint>,
    #[account(
        init,
        payer = admin,
        seeds = [VAULT_USDC_SEED],
        bump,
        token::mint = usdc_mint,
        token::authority = global_config_account,
    )]
    pub program_usdc_token_account: Account<'info, TokenAccount>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> Initialize<'info> {
    pub fn apply(
        ctx: &mut Context<Initialize>,
        initial_sqrt_price_x64: u128,
        tick_upper: i32,
        available_for_swap_in_usdc: u64,
        liquidity: u128,
        fee_bps: u16,
    ) -> Result<()> {
        require!(
            ctx.accounts.usdc_mint.decimals == USDC_DECIMALS,
            ErrorCode::InvalidUsdcDecimals
        );

        require!(
            initial_sqrt_price_x64 >= MIN_SQRT_PRICE_X64
                && initial_sqrt_price_x64 <= MAX_SQRT_PRICE_X64,
            ErrorCode::SqrtPriceOutOfBounds
        );
        let initial_tick = tick_index_from_sqrt_price(&initial_sqrt_price_x64);
        require!(initial_tick < tick_upper, ErrorCode::InvalidTickIndex);

        ctx.accounts.global_config_account.admin = ctx.accounts.admin.key();
        ctx.accounts.global_config_account.usdc_mint = ctx.accounts.usdc_mint.key();
        ctx.accounts.global_config_account.walien_mint = None;
        ctx.accounts.global_config_account.is_sale_active = false;
        ctx.accounts.global_config_account.is_claim_active = false;
        ctx.accounts.global_config_account.bump = ctx.bumps.global_config_account;
        ctx.accounts.global_config_account.fee_bps = fee_bps;
        ctx.accounts.global_config_account.initial_sqrt_price_x64 = initial_sqrt_price_x64;
        ctx.accounts.global_config_account.tick_upper = tick_upper;
        ctx.accounts
            .global_config_account
            .available_for_swap_in_usdc = available_for_swap_in_usdc;
        ctx.accounts.global_config_account.possition_index = 1;
        ctx.accounts.global_config_account.liqudity = liquidity;

        Ok(())
    }
}
