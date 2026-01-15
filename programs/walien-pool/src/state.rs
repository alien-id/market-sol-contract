use anchor_lang::prelude::*;

#[account]
#[derive(Debug)]
pub struct GlobalConfig {
    pub admin: Pubkey,
    pub usdc_mint: Pubkey,
    pub walien_mint: Option<Pubkey>,
    // Flags
    pub is_sale_active: bool,
    pub is_claim_active: bool,

    pub available_for_swap_in_usdc: u64,

    pub possition_index: u64,
    pub bump: u8,
    ///cheeper then recalculate
    // pool config
    pub tick_upper: i32,
    pub fee_bps: u16,
    pub liqudity: u128,
    pub initial_sqrt_price_x64: u128,
}

impl GlobalConfig {
    pub const SIZE: usize = 8 + std::mem::size_of::<Self>();
}

#[account]
#[derive(Debug)]
pub struct UserPosition {
    pub authority: Pubkey,
    pub index: u64,
    pub usdc_spent: u64,
    pub walien_allocation: u64,
    pub last_buy_timestamp: i64,
}
impl UserPosition {
    pub const SIZE: usize = 8 + std::mem::size_of::<Self>();
}

#[account]
#[derive(Debug)]
pub struct UserSummary {
    pub authority: Pubkey,
    pub total_usdc_locked: u64,
    pub total_walien_alloc: u64,
    pub last_buy_timestamp: i64,
    pub global_index_position: u64,
    pub index_position: u64,
}
impl UserSummary {
    pub const SIZE: usize = 8 + std::mem::size_of::<Self>();
}
