use anchor_lang::prelude::*;
use anchor_spl::token_2022::{Token2022, TransferChecked, transfer_checked};
use anchor_spl::token_interface::{
    Mint as Token2022Mint, TokenAccount as Token2022TokenAccount,
};

use crate::{
    constants::{CONFIG_SEED, VAULT_WALIEN_SEED},
    state::GlobalConfig,
};

#[derive(Accounts)]
pub struct WithdrawWalien<'info> {
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
    pub walien_mint: InterfaceAccount<'info, Token2022Mint>,
    #[account(
        mut,
        associated_token::mint = walien_mint,
        associated_token::authority = admin,
        associated_token::token_program = token_program,
    )]
    pub admin_walien_token_account: InterfaceAccount<'info, Token2022TokenAccount>,
    #[account(
        mut,
        seeds = [VAULT_WALIEN_SEED],
        bump,
        token::mint = walien_mint,
        token::authority = global_config_account,
        token::token_program = token_program,
    )]
    pub program_walien_token_account: InterfaceAccount<'info, Token2022TokenAccount>,
    pub token_program: Program<'info, Token2022>,
}

impl<'info> WithdrawWalien<'info> {
    pub fn apply(ctx: &mut Context<WithdrawWalien>) -> Result<()> {
        let balance = ctx.accounts.program_walien_token_account.amount;

        let seeds = &[
            CONFIG_SEED.as_ref(),
            &[ctx.accounts.global_config_account.bump],
        ];
        let signer = &[&seeds[..]];
        let cpi_accounts = TransferChecked {
            from: ctx.accounts.program_walien_token_account.to_account_info(),
            to: ctx.accounts.admin_walien_token_account.to_account_info(),
            authority: ctx.accounts.global_config_account.to_account_info(),
            mint: ctx.accounts.walien_mint.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let transfer_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        transfer_checked(transfer_ctx, balance, ctx.accounts.walien_mint.decimals)?;

        Ok(())
    }
}
