# SPL Token Integration

**Official:** [Token Program](https://quasar-lang.com/docs/spl-tokens/token-program) · [Token-2022](https://quasar-lang.com/docs/spl-tokens/token-2022) · [ATA](https://quasar-lang.com/docs/spl-tokens/associated-token-accounts)

Quasar SPL support lives in `quasar-spl`. Zero-copy views over SPL Token and Token-2022 accounts.

## Dependencies

```toml
[dependencies]
quasar-lang = { path = "../../lang" }
quasar-spl = { path = "../../spl" }
```

```rust
use quasar_spl::{Mint, Token, TokenCpi};
```

## Account types

| Type | Validates | Use as |
|------|-----------|--------|
| `Token` | SPL Token program owner, token account layout | `Account<Token>` |
| `Mint` | SPL Token program owner, mint layout | `Account<Mint>` |
| `InterfaceAccount<Token>` | SPL Token **or** Token-2022 owner | Multi-program token accounts |
| `InterfaceAccount<Mint>` | SPL Token **or** Token-2022 owner | Multi-program mints |
| `Program<TokenProgram>` | Executable token program ID | CPI target |

## Init token account

```rust
#[derive(Accounts)]
pub struct InitToken {
    #[account(mut)]
    pub payer: Signer,
    #[account(
        init,
        payer = payer,
        token(mint = mint, authority = owner, token_program = token_program),
    )]
    pub token_account: Account<Token>,
    pub mint: Account<Mint>,
    pub owner: Signer,
    pub token_program: Program<TokenProgram>,
    pub system_program: Program<SystemProgram>,
    pub rent: Sysvar<Rent>,
}
```

## Init associated token account

```rust
#[account(
    init(idempotent),
    payer = payer,
    associated_token(mint = mint, authority = owner, token_program = token_program),
)]
pub ata: Account<Token>,
```

Requires associated token program account in the struct (see `examples/escrow/`).

## Init mint

```rust
#[account(
    init,
    payer = payer,
    mint(decimals = 6, authority = mint_authority, token_program = token_program),
)]
pub mint: Account<Mint>,
```

## Token CPI via TokenCpi trait

```rust
impl Make {
    pub fn deposit_tokens(&mut self, amount: u64) -> Result<(), ProgramError> {
        self.token_program
            .transfer(&self.maker_ta_a, &self.vault_ta_a, &self.maker, amount)
            .invoke()
    }
}
```

Available methods on `Program<TokenProgram>` (via `TokenCpi`):
- `transfer(from, to, authority, amount)`
- `transfer_checked(from, mint, to, authority, amount, decimals)`
- `mint_to(mint, to, authority, amount)`
- `burn(from, mint, authority, amount)`
- `approve(source, delegate, owner, amount)`
- `revoke(source, owner)`
- `close_account(account, destination, authority)`
- `set_authority(account, current_authority, new_authority, authority_type)`

## PDA-signed token transfer

When token authority is a PDA, use the `#[derive(Accounts)]`-generated seed helper (includes prefix, dynamic seeds, and bump slice):

```rust
let seeds = self.escrow_seeds(bumps);

self.token_program
    .transfer(&self.vault_ta_a, &self.taker_ta_a, &self.escrow, amount)
    .invoke_signed(&seeds)?;
```

Canonical: `examples/escrow/src/instructions/take.rs` (`withdraw_tokens_and_close`). Do not pass `Escrow::seeds(maker)` to `invoke_signed` — that is `#[account]` constraint syntax, not a runtime seed array.

## Closing token accounts

Token accounts are owned by the SPL Token program, not yours. Use the `TokenClose` trait (not `Account::close()`):

```rust
use quasar_spl::TokenClose;

self.vault_ta_a
    .close(self.token_program, self.taker, self.escrow)
    .invoke_signed(&seeds)?;
```

## Token-2022 / interface accounts

For programs accepting both token programs:

```rust
pub token_account: InterfaceAccount<Token>,
pub mint: InterfaceAccount<Mint>,
pub token_program: InterfaceAccount<Token>,  // program selector
```

Compile-time checks enforce token/mint program selector accounts when using interface types.

## Validation-only instructions

Test programs in `tests/programs/test-token-validate/` demonstrate constraint combinations:
- `validate_mint_with_freeze_check`
- `validate_token_2022_check`
- `validate_mint_interface_check`

## Escrow pattern (canonical)

See `examples/escrow/src/instructions/`:

1. **Make** — init escrow PDA, init vault TA with escrow as authority, transfer tokens to vault
2. **Take** — taker sends mint B, PDA signs transfer of mint A from vault, close accounts
3. **Refund** — maker reclaims mint A, close escrow

Key constraint:

```rust
#[account(init(idempotent), payer = maker, token(mint = mint_a, authority = escrow, token_program = token_program))]
pub vault_ta_a: Account<Token>,
```

Escrow PDA must be the token authority for the vault.

## Common mistakes

| Mistake | Fix |
|---------|-----|
| Token authority is signer, not PDA | Use `invoke_signed` with escrow seeds |
| Wrong mint on token account | Add `token(mint = mint, ...)` behavior group |
| Missing token program account | Include `Program<TokenProgram>` in Accounts struct |
| Token-2022 account with `Account<Token>` only | Use `InterfaceAccount<Token>` |
