use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount},
};

use crate::{
    constants::{CONFIG_SEED, USER_SUMMARY_SEED, VAULT_USDC_SEED},
    errors::ErrorCode,
    state::{GlobalConfig, UserPosition, UserSummary},
};

#[derive(Accounts)]
#[instruction(possition_index: u64)]
pub struct RollbackPosition<'info> {
    #[account(
        mut,
        seeds = [
            CONFIG_SEED
        ],
        bump
    )]
    pub global_config_account: Account<'info, GlobalConfig>,
    #[account(
        mut,
        constraint = global_config_account.admin == admin.key())]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [
            global_config_account.key().as_ref(),
            possition_index.to_le_bytes().as_ref()
        ],
        bump,
    )]
    pub user_account: Account<'info, UserPosition>,
    /// CHECK: PDA authority of position
    #[account(
        mut,
        constraint = user_account.authority == user.key()
    )]
    pub user: AccountInfo<'info>,
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
    #[account(
        mut,
        seeds = [USER_SUMMARY_SEED, user.key().as_ref()],
        bump,
        constraint = user_summary.authority == user.key()
    )]
    pub user_summary: Account<'info, UserSummary>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> RollbackPosition<'info> {
    pub fn apply(ctx: &mut Context<RollbackPosition>, _possition_index: u64) -> Result<()> {
        require!(
            ctx.accounts.program_usdc_token_account.amount > 0,
            ErrorCode::WithdrawNotAllowed
        );

        let usdc_amount = ctx.accounts.user_account.usdc_spent;
        let walien_amount = ctx.accounts.user_account.walien_allocation;

        ctx.accounts
            .global_config_account
            .available_for_swap_in_usdc += usdc_amount;

        ctx.accounts.user_summary.total_usdc_locked = ctx
            .accounts
            .user_summary
            .total_usdc_locked
            .checked_sub(usdc_amount)
            .ok_or(ErrorCode::UserSummaryUnderflow)?;
        ctx.accounts.user_summary.total_walien_alloc = ctx
            .accounts
            .user_summary
            .total_walien_alloc
            .checked_sub(walien_amount)
            .ok_or(ErrorCode::UserSummaryUnderflow)?;

        let cpi_accounts = token::Transfer {
            from: ctx.accounts.program_usdc_token_account.to_account_info(),
            to: ctx.accounts.user_usdc_ata.to_account_info(),
            authority: ctx.accounts.global_config_account.to_account_info(),
        };
        let seeds = &[
            CONFIG_SEED.as_ref(),
            &[ctx.accounts.global_config_account.bump],
        ];
        let signer = &[&seeds[..]];
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let transfer_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);

        token::transfer(transfer_ctx, usdc_amount)?;

        ctx.accounts.user_account.walien_allocation = 0;

        ctx.accounts
            .user_account
            .close(ctx.accounts.user.to_account_info())?;

        Ok(())
    }
}
