use crate::{
    constants::{CONFIG_SEED, USER_SUMMARY_SEED, VAULT_USDC_SEED, VAULT_WALIEN_SEED},
    errors::ErrorCode,
    events::ClaimEvent,
    state::{GlobalConfig, UserPosition, UserSummary},
};
use anchor_lang::prelude::program_pack::Pack;
use anchor_lang::prelude::*;
use anchor_spl::token::spl_token::state::Account as SplTokenAccount;
use anchor_spl::{
    associated_token::{create, AssociatedToken, Create},
    token::{self, Mint, Token, TokenAccount},
};

#[derive(Accounts)]
#[instruction(possition_index: u64)]
pub struct Claim<'info> {
    #[account(
        mut,
        seeds = [
            CONFIG_SEED
        ],
        bump
    )]
    pub global_config_account: Account<'info, GlobalConfig>,
    /// CHECK: Admin constrained by address
    #[account(address = global_config_account.admin)]
    pub admin: AccountInfo<'info>,
    #[account(
        constraint = global_config_account.usdc_mint == usdc_mint.key()
    )]
    pub usdc_mint: Account<'info, Mint>,
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
        associated_token::mint = usdc_mint,
        associated_token::authority = admin
    )]
    pub admin_usdc_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub caller: Signer<'info>,

    #[account(
        mut,
        seeds = [
            global_config_account.key().as_ref(),
            possition_index.to_le_bytes().as_ref()
        ],
        bump,
    )]
    pub user_account: Account<'info, UserPosition>,
    #[account(
        constraint = global_config_account.walien_mint == Some(walien_mint.key())
    )]
    pub walien_mint: Account<'info, Mint>,
    #[account(
        mut,
        seeds = [VAULT_WALIEN_SEED],
        bump,
        token::mint = walien_mint,
        token::authority = global_config_account,
    )]
    pub program_walien_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [USER_SUMMARY_SEED, user.key().as_ref()],
        bump,
        constraint = user_summary.authority == user.key()
    )]
    pub user_summary: Account<'info, UserSummary>,
    /// CHECK: We know, that it is walien buyer, constraint = user_account.authority == user.key()
    #[account(
        mut,
        constraint = user_account.authority == user.key()
    )]
    pub user: AccountInfo<'info>,
    /// CHECK: ATA may or may not exist
    #[account(mut)]
    pub user_walien_token_account: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Claim<'info> {
    pub fn apply(ctx: &mut Context<Claim>, _possition_index: u64) -> Result<()> {
        require!(
            ctx.accounts.global_config_account.is_claim_active,
            ErrorCode::ClaimIsNotActive
        );
        require!(
            ctx.accounts.user_account.walien_allocation > 0,
            ErrorCode::NothingToClaim
        );
        let walien_amount = ctx.accounts.user_account.walien_allocation;
        let usdc_spent = ctx.accounts.user_account.usdc_spent;
        let position_index = ctx.accounts.user_account.index;
        let user_position = ctx.accounts.user_account.key();
        let ata_already_exists = ctx.accounts.user_walien_token_account.owner == &token::ID
            && !ctx.accounts.user_walien_token_account.data_is_empty();

        if ata_already_exists {
            let user_walien_ai = ctx.accounts.user_walien_token_account.to_account_info();

            let token_account = SplTokenAccount::unpack(&user_walien_ai.try_borrow_data()?)?;

            require!(
                token_account.mint == ctx.accounts.walien_mint.key()
                    && token_account.owner == ctx.accounts.user.key(),
                ErrorCode::InvalidWalienTokenAccount
            );
        }

        let seeds = &[
            CONFIG_SEED.as_ref(),
            &[ctx.accounts.global_config_account.bump],
        ];
        let signer = &[&seeds[..]];

        if !ata_already_exists {
            let cpi_accounts = Create {
                payer: ctx.accounts.caller.to_account_info(),
                associated_token: ctx.accounts.user_walien_token_account.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
                mint: ctx.accounts.walien_mint.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
            };
            create(CpiContext::new(
                ctx.accounts.associated_token_program.to_account_info(),
                cpi_accounts,
            ))?;
        }

        {
            let cpi_accounts = token::Transfer {
                from: ctx.accounts.program_walien_token_account.to_account_info(),
                to: ctx.accounts.user_walien_token_account.to_account_info(),
                authority: ctx.accounts.global_config_account.to_account_info(),
            };
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let transfer_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);

            token::transfer(transfer_ctx, ctx.accounts.user_account.walien_allocation)?;
        }

        {
            let cpi_accounts = token::Transfer {
                from: ctx.accounts.program_usdc_token_account.to_account_info(),
                to: ctx.accounts.admin_usdc_token_account.to_account_info(),
                authority: ctx.accounts.global_config_account.to_account_info(),
            };
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let transfer_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);

            token::transfer(transfer_ctx, usdc_spent)?;
        }

        emit!(ClaimEvent {
            caller: ctx.accounts.caller.key(),
            user: ctx.accounts.user.key(),
            user_position,
            position_index,
            walien_amount,
        });

        ctx.accounts.user_summary.total_walien_alloc = ctx
            .accounts
            .user_summary
            .total_walien_alloc
            .checked_sub(walien_amount)
            .ok_or(ErrorCode::UserSummaryUnderflow)?;

        ctx.accounts.user_summary.total_usdc_locked = ctx
            .accounts
            .user_summary
            .total_usdc_locked
            .checked_sub(usdc_spent)
            .ok_or(ErrorCode::UserSummaryUnderflow)?;

        let should_close_user_summary = ctx.accounts.user_summary.total_walien_alloc == 0
            && ctx.accounts.user_summary.total_usdc_locked == 0;

        let recipient = if ata_already_exists {
            ctx.accounts.user.to_account_info()
        } else {
            ctx.accounts.caller.to_account_info()
        };

        ctx.accounts.user_account.walien_allocation = 0;

        ctx.accounts.user_account.close(recipient.clone())?;

        if should_close_user_summary {
            ctx.accounts.user_summary.close(recipient)?;
        }

        Ok(())
    }
}
