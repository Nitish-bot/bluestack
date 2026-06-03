# Anchor Migration Guide

**Official:** [Migrating from Anchor](https://quasar-lang.com/docs/getting-started/migrating-from-anchor)

Step-by-step mapping from Anchor patterns to Quasar.

> **Greenfield programs:** Skip this file if you are not porting Anchor code — use [SKILL.md](../SKILL.md) workflow and [canonical-examples.md](canonical-examples.md) instead.

## Official quick reference

| Anchor | Quasar |
|--------|--------|
| `anchor_lang::prelude::*` | `quasar_lang::prelude::*` |
| `Context<T>` | `Ctx<T>` |
| `Result<()>` | `Result<(), ProgramError>` |
| `Signer<'info>` | `Signer` (no lifetime param on struct) |
| `#[account(mut)]` | `#[account(mut)]` on the field — keep `Account<T>` / `Signer`, not `&'info mut` |
| `#[account]` | `#[account(discriminator = N)]` (non-zero) |
| Discriminator optional (SHA-256) | `#[instruction(discriminator = N)]` required |
| `CpiContext::new(...)` | `.transfer(...).invoke()` |
| Manual PDA seeds | `bumps.name_seeds()` auto-generated |
| `String`/`Vec` (heap) | `String<MAX>` / `Vec<T, MAX>` (Pod aliases in prelude; zero-copy, no lifetime) |

Logic lives in `impl` methods on the accounts struct; the `#[program]` block is dispatch only.

## Crate setup

| Anchor | Quasar |
|--------|--------|
| `anchor-lang = "0.30"` | `quasar-lang = { path = "../../lang" }` |
| `anchor-spl = "0.30"` | `quasar-spl = { path = "../../spl" }` |
| `#![allow(unexpected_cfgs)]` | `#![no_std]` |
| `use anchor_lang::prelude::*` | `use quasar_lang::prelude::*` |
| `declare_id!` from anchor | `declare_id!` from `solana_address` (via prelude) |

## Program module

```rust
// Anchor
#[program]
pub mod my_program {
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> { }
}

// Quasar
#[program]
mod my_program {
    use super::*;
    #[instruction(discriminator = 0)]
    pub fn initialize(ctx: Ctx<Initialize>) -> Result<(), ProgramError> { }
}
```

**Changes:**
- Add `#[instruction(discriminator = N)]` to every handler
- `Context<T>` → `Ctx<T>`
- `Result<()>` → `Result<(), ProgramError>`
- Handlers inside private mod (not `pub mod`)

## Account types

```rust
// Anchor
#[account]
pub struct Counter {
    pub authority: Pubkey,
    pub count: u64,
}

// Quasar (non-PDA fields only shown — PDA accounts need #[seeds] + bump: u8)
#[account(discriminator = 1, set_inner)]
#[seeds(b"counter", authority: Address)]
pub struct Counter {
    pub authority: Address,
    pub count: u64,
    pub bump: u8,
}
```

**Changes:**
- Explicit `discriminator` on every account type
- PDA state: `#[seeds(...)]` on type, `Type::seeds(...)`, stored `bump` — see [canonical-examples.md](canonical-examples.md)
- `Pubkey` → `Address`
- Consider `PodU64` for fields needing explicit overflow control in shared state

## Init

```rust
// Anchor
#[account(
    init,
    payer = user,
    space = 8 + Counter::INIT_SPACE,
)]
pub counter: Account<Counter>,

// Quasar — omit space =; Space::SPACE includes discriminator; PDA init needs typed seeds + bump
#[account(discriminator = 1, set_inner)]
#[seeds(b"counter", user: Address)]
pub struct Counter { pub authority: Address, pub count: u64, pub bump: u8 }

#[account(mut, init, payer = user, address = Counter::seeds(user.address()))]
pub counter: Account<Counter>,
```

Do not add Anchor's `8 +` discriminator prefix. Quasar's `Space::SPACE` already counts the account discriminator bytes.

Use `set_inner(CounterInner { ... bump: bumps.counter })` on init (see `examples/escrow`, `assets/minimal-counter-template.rs`).

## Seeds / PDA

```rust
// Anchor
#[account(
    seeds = [b"vault", user.key().as_ref()],
    bump,
)]
pub vault: Account<Vault>,

// Quasar (typed PDA via address =)
#[account(discriminator = 1)]
#[seeds(b"vault", user: Address)]
pub struct Vault { pub bump: u8 }

#[account(address = Vault::seeds(user.address()))]
pub vault: Account<Vault>,

// Quasar PDA init (examples/multisig/src/instructions/create.rs)
#[account(init, payer = creator, address = MultisigConfig::seeds(creator.address()))]
pub config: Account<MultisigConfig>,
```

Typed PDA init/validate uses `address = Type::seeds(args)` — not `seeds =` or bare `bump` (removed; compile-fail). Fixed pubkey checks use `address = CONST` without `::seeds`.

Store bump in account struct — Anchor often uses `bump` in seeds constraint only; Quasar recommends persisting it.

## Constraints

Most Anchor constraints map 1:1:

| Anchor | Quasar |
|--------|--------|
| `has_one = x` | `has_one(x)` |
| `constraint = expr` | `constraints(expr)` |
| `address = x` | `address = x` (PDA: `address = Type::seeds(arg.address())`) |
| `token::mint = m` | `token(mint = m, authority = ..., token_program = ...)` |
| `associated_token::...` | `associated_token(mint = ..., authority = ..., ...)` |
| `init_if_needed` | `init(idempotent)` |
| `close = dest` | `close(dest = dest)` |
| `@ ErrorCode` | `@ ElectionError` |

## SPL CPI

```rust
// Anchor
token::transfer(
    CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer { from, to, authority },
    ),
    amount,
)?;

// Quasar
ctx.accounts.token_program
    .transfer(&from, &to, &authority, amount)
    .invoke()?;
```

No `CpiContext`. PDA-signed CPI uses the generated accounts helper (not `Type::seeds(...)` — that is constraint syntax only):

```rust
let seeds = self.vault_seeds(&ctx.bumps);  // or vault_signer(bumps) for #[derive(Seeds)] fields
self.token_program.transfer(...).invoke_signed(&seeds)?;
```

See `examples/escrow/src/instructions/take.rs` (`self.escrow_seeds(bumps)`).

## Errors

```rust
// Anchor
#[error_code]
pub enum ElectionError { #[msg("...")] Unauthorized }

// Quasar — plain variants (tests/programs/test-errors/src/errors.rs)
#[error_code]
pub enum ElectionError {
    Unauthorized = 0,
    InsufficientFunds,
}
```

No `#[msg]` attribute. Add explicit discriminants starting at 0 for clarity.

## Events

```rust
// Anchor
#[event]
pub struct SwapEvent { pub amount: u64 }

// Quasar
#[event(discriminator = 0)]
pub struct SwapEvent { pub amount: u64 }

emit!(SwapEvent { amount });
```

## Context bumps

```rust
// Anchor
ctx.bumps.vault

// Quasar
ctx.bumps.vault  // same, on Ctx<T>
```

## Remaining accounts

```rust
// Anchor
ctx.remaining_accounts

// Quasar
ctx.remaining_accounts()  // on CtxWithRemaining<T>
```

Switch handler to `CtxWithRemaining` when needed.

## AccountInfo / UncheckedAccount

`UncheckedAccount` in Quasar ≈ Anchor's `UncheckedAccount`. No direct `AccountInfo` in handlers — use account wrapper types.

## Init_if_needed

Supported with same semantics. Validate existing account layout on second call.

## Realloc

Anchor `realloc` → Quasar `realloc` + `realloc::payer`. Requires `alloc` feature.

## Things Anchor has that Quasar handles differently

| Anchor feature | Quasar approach |
|----------------|-----------------|
| Auto-discriminator | Explicit `discriminator = N` |
| `AccountLoader` | Direct ZC access via `Account<T>` |
| `Interface` accounts | `InterfaceAccount<T>` |
| `ProgramData` | Not needed — upgrade authority checks manual |
| IDL built into macro | Separate `quasar-idl` crate |
| `#[access_control]` | Use constraint attrs or manual checks in handler |
| `Box<Account>` | Not used — `Account<T>` with `#[account(mut)]` when writable |

## Migration workflow

1. Create parallel Quasar crate with same instruction surface
2. Port `state` structs — add discriminators, fix field types
3. Port `Accounts` structs — verify constraint syntax
4. Port handlers — thin wrappers calling impl methods
5. Port tests to `quasar-svm` (replace `ProgramTest` / `BanksClient`)
6. Compare CU with examples in repo

## Client / SDK

Anchor TS client → generate Rust client via `quasar-idl` or hand-write instruction builders (see `examples/*/client/`).

Do not assume `@coral-xyz/anchor` compatibility.
