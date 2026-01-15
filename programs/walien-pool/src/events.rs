use anchor_lang::prelude::*;

#[event]
pub struct BuyEvent {
    pub user: Pubkey,
    pub user_position: Pubkey,
    pub position_index: u64,
    pub usdc_amount: u64,
    pub walien_amount: u64,
    pub price_after: u128,
}

#[event]
pub struct ClaimEvent {
    pub caller: Pubkey,
    pub user: Pubkey,
    pub user_position: Pubkey,
    pub position_index: u64,
    pub walien_amount: u64,
}
