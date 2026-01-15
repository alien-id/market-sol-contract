use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

use crate::constants::WALIEN_DECIMALS;
use crate::{
    constants::{CONFIG_SEED, VAULT_WALIEN_SEED},
    errors::ErrorCode,
    state::GlobalConfig,
};

#[derive(Accounts)]
pub struct SetWalien<'info> {
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
    #[account()]
    pub walien_mint: Account<'info, Mint>,
    #[account(
        init,
        payer = admin,
        seeds = [VAULT_WALIEN_SEED],
        bump,
        token::mint = walien_mint,
        token::authority = global_config_account,
    )]
    pub program_walien_token_account: Account<'info, TokenAccount>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> SetWalien<'info> {
    pub fn apply(ctx: &mut Context<SetWalien>) -> Result<()> {
        require!(
            ctx.accounts.walien_mint.decimals == WALIEN_DECIMALS,
            ErrorCode::InvalidWalienDecimals
        );

        ctx.accounts.global_config_account.walien_mint = Some(ctx.accounts.walien_mint.key());

        Ok(())
    }
}
