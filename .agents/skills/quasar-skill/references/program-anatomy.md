# Program Anatomy

**Official:** [Program Structure](https://quasar-lang.com/docs/core-concepts/program-structure)

Structure of a Quasar on-chain program crate.

## Minimal crate layout

```
my-program/
├── Cargo.toml
├── src/
│   ├── lib.rs           # declare_id, #[program], mod declarations
│   ├── state.rs         # #[account] types (optional split)
│   ├── events.rs        # #[event] types (optional)
│   ├── errors.rs        # #[error_code] enum (optional)
│   ├── instructions/
│   │   ├── mod.rs
│   │   ├── deposit.rs   # #[derive(Accounts)] + impl methods
│   │   └── withdraw.rs
│   └── tests.rs         # quasar-svm tests (#[cfg(test)])
└── client/              # optional: instruction builders for tests/off-chain
    ├── Cargo.toml
    └── src/
```

## lib.rs skeleton

```rust
#![no_std]

use quasar_lang::prelude::*;

mod instructions;
use instructions::*;
mod state;
mod events;

#[cfg(test)]
mod tests;

declare_id!("YourProgramId111111111111111111111111111111");

#[program]
mod my_program {
    use super::*;

    #[instruction(discriminator = 0)]
    pub fn initialize(ctx: Ctx<Initialize>) -> Result<(), ProgramError> {
        ctx.accounts.handler()
    }
}
```

## Cargo.toml

```toml
[package]
name = "my-program"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "lib"]

[features]
default = []
alloc = []      # PodString, PodVec, heap instructions
debug = []      # overflow panics, debug logging
client = []     # expose types for off-chain client crate

[dependencies]
quasar-lang = { path = "../../lang" }
quasar-spl = { path = "../../spl", optional = true }

[dev-dependencies]
quasar-svm = { git = "https://github.com/blueshift-gg/quasar-svm" }
my-program-client = { path = "client" }
```

## Entrypoint (generated)

The `#[program]` macro emits:

1. **`dispatch!`** — reads ix discriminator, routes to handler, passes `Ctx`
2. **`no_alloc!`** — default: heap disabled globally
3. **`panic_handler!`** — abort on panic in release

Per-instruction `heap` attribute selectively re-enables allocation for that arm.

## Module responsibilities

| Module | Contains |
|--------|----------|
| `state.rs` | `#[account]` structs, `#[seeds(...)]`, `Space` constants |
| `instructions/*.rs` | `#[derive(Accounts)]` structs + `impl` with business logic |
| `events.rs` | `#[event(discriminator = N)]` structs |
| `errors.rs` | `#[error_code]` enum |

## Instruction handler patterns

**Thin handler** (logic in accounts impl):

```rust
#[instruction(discriminator = 0)]
pub fn make(ctx: Ctx<Make>, deposit: u64, receive: u64) -> Result<(), ProgramError> {
    ctx.accounts.make_escrow(receive, &ctx.bumps)?;
    ctx.accounts.deposit_tokens(deposit)
}
```

**Direct mutation** (simple counters):

```rust
#[instruction(discriminator = 0)]
pub fn increment(ctx: Ctx<Increment>) -> Result<(), ProgramError> {
    ctx.accounts.counter.count += 1;
    Ok(())
}
```

## Accounts struct field types

Use owned wrapper types on `#[derive(Accounts)]` fields. Mark writable accounts with `#[account(mut)]` — not Rust `mut` on the type or `&'info` references:

```rust
#[derive(Accounts)]
pub struct Increment {
    #[account(mut, has_one(authority), address = Counter::seeds(authority.address()))]
    pub counter: Account<Counter>,
    pub authority: Signer,
}
```

Business logic that mutates multiple accounts often uses `impl Increment { fn handler(&mut self, ...) }` (see `examples/escrow/`). `Ctx<T>` still parses accounts from the SVM input buffer at runtime.

## Client crate (testing / off-chain)

The `quasar-idl` crate generates typed instruction builders. Example pattern in `examples/vault/client/`:

```rust
// In tests:
let ix: Instruction = DepositInstruction {
    user,
    vault,
    system_program,
    amount,
}.into();
```

Client crates compile on host targets; program crate is `no_std` only.

## Features

| Feature | Effect |
|---------|--------|
| `alloc` | Enables `alloc` crate in program; required for dynamic account fields and heap ix |
| `debug` | Panic on Pod arithmetic overflow; richer logging |
| `client` | Expose account state types for client generation |

## IDL output

Building produces IDL JSON under `target/idl/`. Used by:

- `declare_program!` for CPI into this program from another
- Client codegen
- External tooling (see quasar-lang.com — not primary skill workflow)
