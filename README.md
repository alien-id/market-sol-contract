# wALIEN market pool

On-chain Anchor program that sells wALIEN tokens in exchange for a pay token (USDC), using Orca Whirlpool-style pricing math for dynamic price movement.
The program does not mint immediately spendable tokens — instead, users buy virtual wALIEN tokens, which are recorded on-chain and can be claimed later when the claim phase is activated.

## Program Overview
- Main entrypoint: `programs/walien-pool/src/lib.rs`.
- Core math: `programs/walien-pool/src/orca_math` (see its README for derivations).
- Seeds: `config` (global state PDA), `vault_usdc` (USDC ATA owned by program PDA), `vault_walien` (wALIEN ATA owned by program PDA).

## Accounts
- `GlobalConfig` (`programs/walien-pool/src/state.rs`)
  - Admin pubkey, USDC mint, optional wALIEN mint.
  - Flags: `is_sale_active`, `is_claim_active`.
  - Pool state: `available_for_swap_in_usdc`, `tick_upper`, `fee_bps`, `liqudity`, `initial_sqrt_price_x64`, `possition_index`, `bump`.
- `UserPosition`
  - Per-purchase record with `authority`, `index`, `usdc_spent`, `walien_allocation`, `last_buy_timestamp`.

## Instructions (Admin)
- `initialize(initial_sqrt_price_x64, tick_upper, available_for_swap_in_usdc, liquidity)`  
  Creates the `GlobalConfig` PDA and the USDC vault ATA. Sets sale/claim flags off, wALIEN mint unset, fee to zero, and starts `possition_index` at 1.
- `set_walien(walien_mint)`  
  Stores the wALIEN mint on `GlobalConfig` and creates the wALIEN vault ATA.
- `deposit_walien(amount)`  
  Transfers wALIEN from the admin into the program vault, then drains the accumulated USDC vault balance back to the admin.
- `set_sale_activity(is_active)`  
  Toggles `is_sale_active`.
- `set_claim_activity(is_active)`  
  Requires wALIEN mint to be set; toggles `is_claim_active`.
- `transfer_admin_authority(new_admin)`  
  Updates the admin pubkey on `GlobalConfig`.

## Instructions (User)
- `quote(amount: u64) -> u64`  
  Read-only view that returns the expected wALIEN out for the given USDC in, based on current price/liquidity and fee settings.
- `buy(amount: u64, min_tokens_out: u64)`  
  Requires sale to be active and `available_for_swap_in_usdc >= amount`. Calculates swap output, enforces slippage, transfers USDC into the vault, records a `UserPosition`, advances price/liquidity state, decrements available swap, and increments `possition_index`.
- `claim(possition_index: u64)`  
  Requires claim to be active. Transfers the recorded wALIEN allocation from the vault to the user ATA (creates it if missing) and closes the `UserPosition`. Rent refund goes to the caller on first claim into a fresh ATA, otherwise to the recorded user.

## PDA/ATA Map
- Global config: `["config"]`
- User position: `[global_config, possition_index_le_bytes]`
- USDC vault ATA: `["vault_usdc"]` owned by `global_config`
- wALIEN vault ATA: `["vault_walien"]` owned by `global_config`

## 🔄 Flow to Use the Program

Follow this sequence when interacting with the contract:

1. **Initialize**  
   Set up all required accounts and global configuration.

2. **Set Sale Available**  
   Enable the sale mode so users can start buying.

3. **Buy**  
   Execute the purchase flow (e.g., buy tokens / wALIEN).

4. **Set wALIEN**  
   Configure the wALIEN mint.

5. **Deposit wALIEN**  
   Admin deposit the wALIEN tokens into the program.

6. **Set Claim Available**  
   Enable the claim phase.

7. **Claim**  
   Users can now claim wALIEN tokens.
