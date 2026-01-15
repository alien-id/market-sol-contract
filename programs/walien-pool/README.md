# Walien Pool Program

On-chain program built with Anchor for selling/buying Walien tokens against a pay token (USDC). The core math is derived from Orca Whirlpools (see `src/orca_math/README.md`).


## Admin Instructions
- `initialize(initial_sqrt_price_x64, tick_upper, available_for_swap_in_usdc, liquidity)`: Creates global config PDA and USDC vault ATA.
- `set_walien(walien_mint)`: Sets Walien mint and creates Walien vault ATA.
- `deposit_walien(amount)`: Admin deposits Walien to vault; drains USDC vault back to admin.
- `set_sale_activity(is_active)`: Toggle sale on/off.
- `set_claim_activity(is_active)`: Toggle claim on/off.
- `transfer_admin_authority(new_admin)`: Change admin pubkey.

## User Instructions
- `quote(amount)`: Read-only view; expected Walien out for a pay amount at current price/liquidity.
- `buy(amount, min_tokens_out)`: User pays USDC, receives Walien allocation; advances price and position index.
- `claim(possition_index, user)`: Transfers allocated Walien from vault to user ATA and closes the position.
## Accounts
- **GlobalConfig**: Admin, mints, flags, liquidity, price, tick bounds, available_for_swap_in_usdc, position index, bump.
- **UserPosition**: Authority, usdc_spent, walien_allocation, last_buy_timestamp.

## PDAs & ATAs (high level)
- Global config: `["config"]`
- User position: `[global_config, position_index_le_bytes]`
- USDC vault: `["vault_usdc"]` (ATA of global config for USDC mint)
- Walien vault: `["vault_walien"]` (ATA of global config for Walien mint)
- User ATAs: standard associated token accounts for user + mint.