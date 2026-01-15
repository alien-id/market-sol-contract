pub mod constants;
pub mod errors;
mod events;
pub mod instructions;
pub mod orca_math;
pub mod state;
pub mod utils;

use anchor_lang::prelude::*;
use instructions::*;

#[cfg(not(feature = "no-entrypoint"))]
use solana_security_txt::security_txt;

#[cfg(not(feature = "no-entrypoint"))]
security_txt! {
    name: "WALIEN Pool Program",
    project_url: "https://alien.org/",
    contacts: "email:aliensol@eti.gg, twitter:@alienorg",
    policy: "https://alien.org/sol-security-policy",
    preferred_languages: "en",
    source_code: "https://github.com/alien-id/market-sol-contract"
}

declare_id!("Hp8rd1zGZdyJNidxx4461e9buwGhab9DcGnsc23wha3");

#[program]
pub mod walien_pool {
    use super::*;

    pub fn initialize(
        mut ctx: Context<Initialize>,
        initial_sqrt_price_x64: u128,
        tick_upper: i32,
        available_for_swap_in_usdc: u64,
        liquidity: u128,
        fee_bps: u16,
    ) -> Result<()> {
        Initialize::apply(
            &mut ctx,
            initial_sqrt_price_x64,
            tick_upper,
            available_for_swap_in_usdc,
            liquidity,
            fee_bps,
        )
    }

    pub fn set_walien(mut ctx: Context<SetWalien>) -> Result<()> {
        SetWalien::apply(&mut ctx)
    }

    pub fn deposit_walien(mut ctx: Context<DepositWalien>, amount: u64) -> Result<()> {
        DepositWalien::apply(&mut ctx, amount)
    }
    pub fn set_sale_activity(mut ctx: Context<SetSaleActivity>, is_active: bool) -> Result<()> {
        SetSaleActivity::apply(&mut ctx, is_active)
    }

    pub fn set_claim_activity(mut ctx: Context<SetClaimActivity>, is_active: bool) -> Result<()> {
        SetClaimActivity::apply(&mut ctx, is_active)
    }

    pub fn transfer_admin_authority(mut ctx: Context<TransferAdminAuthority>) -> Result<()> {
        TransferAdminAuthority::apply(&mut ctx)
    }

    pub fn quote(ctx: Context<Quote>, amount: u64) -> Result<u64> {
        Quote::apply(&ctx, amount)
    }
    pub fn buy(mut ctx: Context<Buy>, amount: u64, min_tokens_out: u64) -> Result<()> {
        Buy::apply(&mut ctx, amount, min_tokens_out)
    }

    pub fn claim(mut ctx: Context<Claim>, possition_index: u64) -> Result<()> {
        Claim::apply(&mut ctx, possition_index)
    }

    pub fn withdraw_usdc(mut ctx: Context<WithdrawUSDC>, possition_index: u64) -> Result<()> {
        WithdrawUSDC::apply(&mut ctx, possition_index)
    }

    pub fn rollback_position(
        mut ctx: Context<RollbackPosition>,
        possition_index: u64,
    ) -> Result<()> {
        RollbackPosition::apply(&mut ctx, possition_index)
    }

    pub fn withdraw_walien(mut ctx: Context<WithdrawWalien>) -> Result<()> {
        WithdrawWalien::apply(&mut ctx)
    }
}
