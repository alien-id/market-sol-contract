use crate::{
    constants::{CONFIG_SEED, USDC_DECIMALS, USER_SUMMARY_SEED, VAULT_USDC_SEED},
    errors::ErrorCode,
    events::BuyEvent,
    state::{GlobalConfig, UserPosition, UserSummary},
    utils::calculate_swap_from_config,
};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct Buy<'info> {
    #[account(
        mut,
        seeds = [
            CONFIG_SEED
        ],
        bump
    )]
    pub global_config_account: Account<'info, GlobalConfig>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        init,
        space = UserPosition::SIZE,
        payer = user,
        seeds = [
            global_config_account.key().as_ref(),
            global_config_account.possition_index.to_le_bytes().as_ref()
        ],
        bump
    )]
    pub user_account: Account<'info, UserPosition>,
    #[account(
        init_if_needed,
        space = UserSummary::SIZE,
        payer = user,
        seeds = [USER_SUMMARY_SEED, user.key().as_ref()],
        bump
    )]
    pub user_summary: Account<'info, UserSummary>,
    #[account(
        constraint = global_config_account.usdc_mint == usdc_mint.key()
    )]
    pub usdc_mint: Account<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = user
    )]
    pub user_usdc_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [VAULT_USDC_SEED],
        bump,
        token::mint = usdc_mint,
        token::authority = global_config_account,
    )]
    pub program_usdc_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> Buy<'info> {
    pub fn apply(ctx: &mut Context<Buy>, amount: u64, min_tokens_out: u64) -> Result<()> {
        require!(
            ctx.accounts.global_config_account.is_sale_active,
            ErrorCode::SaleNotActive
        );

        let cfg = &mut ctx.accounts.global_config_account;

        let calculation_result = calculate_swap_from_config(cfg, amount)?;
        let transfer_amount = calculation_result
            .amount_in
            .checked_add(calculation_result.fee_amount)
            .ok_or(ErrorCode::AmountCalcOverflow)?;

        let max_usdc_cap = 100_000u64
            .checked_mul(10u64.pow(USDC_DECIMALS as u32))
            .ok_or(ErrorCode::AmountCalcOverflow)?;

        require!(transfer_amount <= max_usdc_cap, ErrorCode::UsdcCapExceeded);
        require!(
            cfg.available_for_swap_in_usdc <= max_usdc_cap,
            ErrorCode::UsdcCapExceeded
        );

        require!(
            cfg.available_for_swap_in_usdc >= transfer_amount,
            ErrorCode::InsufficientAvailableForSwap
        );

        require!(
            calculation_result.amount_out > 0,
            ErrorCode::ZeroTradableAmount
        );
        require!(
            calculation_result.amount_out >= min_tokens_out,
            ErrorCode::SlippageExceeded
        );

        let position_index = cfg.possition_index;
        let user_index_position = ctx.accounts.user_summary.index_position;

        {
            let cpi_accounts = token::Transfer {
                from: ctx.accounts.user_usdc_ata.to_account_info(),
                to: ctx.accounts.program_usdc_token_account.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            };
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let transfer_ctx = CpiContext::new(cpi_program, cpi_accounts);

            token::transfer(transfer_ctx, transfer_amount)?;
        }

        ctx.accounts.user_account.authority = ctx.accounts.user.key();

        let now_ts = Clock::get()?.unix_timestamp;
        ctx.accounts.user_account.last_buy_timestamp = now_ts;
        ctx.accounts.user_account.usdc_spent += transfer_amount;
        ctx.accounts.user_account.walien_allocation += calculation_result.amount_out;

        ctx.accounts.user_summary.authority = ctx.accounts.user.key();
        ctx.accounts.user_summary.last_buy_timestamp = now_ts;
        ctx.accounts.user_summary.total_usdc_locked = ctx
            .accounts
            .user_summary
            .total_usdc_locked
            .checked_add(transfer_amount)
            .ok_or(ErrorCode::AmountCalcOverflow)?;
        ctx.accounts.user_summary.total_walien_alloc = ctx
            .accounts
            .user_summary
            .total_walien_alloc
            .checked_add(calculation_result.amount_out)
            .ok_or(ErrorCode::AmountCalcOverflow)?;

        cfg.available_for_swap_in_usdc = cfg
            .available_for_swap_in_usdc
            .checked_sub(transfer_amount)
            .ok_or(ErrorCode::InsufficientAvailableForSwap)?;
        cfg.initial_sqrt_price_x64 = calculation_result.next_price;
        ctx.accounts.user_account.index = position_index;
        ctx.accounts.user_summary.global_index_position = position_index;
        ctx.accounts.user_summary.index_position = user_index_position
            .checked_add(1)
            .ok_or(ErrorCode::AmountCalcOverflow)?;
        cfg.possition_index += 1;

        emit!(BuyEvent {
            user: ctx.accounts.user.key(),
            user_position: ctx.accounts.user_account.key(),
            position_index,
            usdc_amount: transfer_amount,
            walien_amount: calculation_result.amount_out,
            price_after: calculation_result.next_price,
        });

        Ok(())
    }
}
