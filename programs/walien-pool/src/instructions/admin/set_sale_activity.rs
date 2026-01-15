use crate::{constants::CONFIG_SEED, state::GlobalConfig};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct SetSaleActivity<'info> {
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
}

impl<'info> SetSaleActivity<'info> {
    pub fn apply(ctx: &mut Context<SetSaleActivity>, sale_is_active: bool) -> Result<()> {
        ctx.accounts.global_config_account.is_sale_active = sale_is_active;
        Ok(())
    }
}
