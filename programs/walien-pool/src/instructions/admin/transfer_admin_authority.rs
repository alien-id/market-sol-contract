use crate::{constants::CONFIG_SEED, state::GlobalConfig};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct TransferAdminAuthority<'info> {
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
    /// CHECK: no need to check here.
    pub new_admin_authority: UncheckedAccount<'info>,
}

impl<'info> TransferAdminAuthority<'info> {
    pub fn apply(ctx: &mut Context<TransferAdminAuthority>) -> Result<()> {
        ctx.accounts.global_config_account.admin = ctx.accounts.new_admin_authority.key();
        Ok(())
    }
}
