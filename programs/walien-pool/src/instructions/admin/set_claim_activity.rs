use crate::{constants::CONFIG_SEED, errors, state::GlobalConfig};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct SetClaimActivity<'info> {
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

impl<'info> SetClaimActivity<'info> {
    pub fn apply(ctx: &mut Context<SetClaimActivity>, claim_is_active: bool) -> Result<()> {
        require!(
            ctx.accounts.global_config_account.walien_mint.is_some(),
            errors::ErrorCode::WalienIsNotSet
        );
        ctx.accounts.global_config_account.is_claim_active = claim_is_active;
        Ok(())
    }
}
