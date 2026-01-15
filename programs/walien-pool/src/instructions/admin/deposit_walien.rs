use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};

use crate::{
    constants::{CONFIG_SEED, VAULT_WALIEN_SEED},
    state::GlobalConfig,
};

#[derive(Accounts)]
pub struct DepositWalien<'info> {
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
        constraint = global_config_account.walien_mint == Some(walien_mint.key())
    )]
    pub walien_mint: Account<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = walien_mint,
        associated_token::authority = admin)]
    pub admin_walien_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [VAULT_WALIEN_SEED],
        bump,
        token::mint = walien_mint,
        token::authority = global_config_account,
    )]
    pub program_walien_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

impl<'info> DepositWalien<'info> {
    pub fn apply(ctx: &mut Context<DepositWalien>, amount: u64) -> Result<()> {
        {
            let cpi_accounts = token::Transfer {
                from: ctx.accounts.admin_walien_token_account.to_account_info(),
                to: ctx.accounts.program_walien_token_account.to_account_info(),
                authority: ctx.accounts.admin.to_account_info(),
            };
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let transfer_ctx = CpiContext::new(cpi_program, cpi_accounts);

            token::transfer(transfer_ctx, amount)?;
        }

        Ok(())
    }
}
