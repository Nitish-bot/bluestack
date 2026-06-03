# Errors, Events, and Remaining Accounts

**Official:** [CPI (events)](https://quasar-lang.com/docs/core-concepts/cpi) · [Instructions (return data)](https://quasar-lang.com/docs/core-concepts/instructions)

## Custom errors

```rust
#[error_code]
pub enum MyError {
    Unauthorized = 0,
    InsufficientFunds,   // auto-incremented code
    Explicit = 100,      // explicit code
}
```

Pattern from `tests/programs/test-errors/src/errors.rs` — plain enum variants only (no `#[msg]` attribute).

Usage in handlers and constraints:

```rust
require!(amount > 0, MyError::InsufficientFunds);
require_eq!(self.counter.authority, *self.authority.address(), MyError::Unauthorized);

#[account(constraints(vault.is_active) @ MyError::VaultFrozen)]
```

### Error code mapping

Custom errors encode as `6000 + discriminant` (Anchor-compatible range). Use `@ Error` on constraints for custom mapping.

### ProgramError

Return `ProgramError` directly for standard variants:

```rust
Err(ProgramError::InvalidAccountData)
Err(ProgramError::ArithmeticOverflow)
```

## Runtime require macros

| Macro | Behavior |
|-------|----------|
| `require!(cond, err)` | Panic/return if false |
| `require_eq!(a, b, err)` | Equality check |
| `require_keys_eq!(a, b, err)` | Address equality |

Prefer constraint attributes over runtime checks when validation depends only on account metadata.

## Events

Define:

```rust
#[event(discriminator = 0)]
pub struct MakeEvent {
    pub escrow: Address,
    pub maker: Address,
    pub deposit: u64,
    pub receive: u64,
}
```

Emit:

```rust
emit!(MakeEvent {
    escrow: *self.escrow.address(),
    maker: *self.maker.address(),
    deposit,
    receive,
});
```

### emit! vs emit_cpi!

| Macro | Mechanism | CU | Indexer compatibility |
|-------|-----------|-----|------------------------|
| `emit!` | `sol_log_data` syscall | Lower | Parses log data |
| `emit_cpi!` | Self-CPI with event ix | Higher | Anchor-style |

Use `emit!` by default. Use `emit_cpi!` when indexers expect self-CPI event pattern.

### Event constraints

- Fields must be supported Pod/fixed types
- No `String` in events unless using supported fixed types
- Tuple structs **not supported** — use named fields
- Explicit discriminator required; same uniqueness rules as accounts

Compile failures: `lang/tests/compile_fail/event_*`

## Return data

```rust
use quasar_lang::prelude::set_return_data;

set_return_data(&result.to_le_bytes());
```

Off-chain clients read return data from transaction metadata.

## Remaining accounts

Default `Ctx<T>` parses only declared accounts. Use `CtxWithRemaining<T>` when the instruction accepts trailing accounts.

### Typed parse (preferred)

Canonical: `examples/multisig/src/lib.rs`

```rust
#[instruction(discriminator = 0)]
pub fn create(ctx: CtxWithRemaining<Create>, threshold: u8) -> Result<(), ProgramError> {
    let signers = ctx.remaining_accounts().parse::<Signer, 10>()?;
    ctx.accounts.create_multisig(threshold, &ctx.bumps, signers)
}
```

`Remaining<T, N>` parses up to `N` accounts of type `T` with duplicate detection and overflow errors (`lang/src/remaining.rs`). Pass the result to `impl` methods — see `examples/multisig/src/instructions/create.rs`.

### Raw iterator (advanced)

```rust
let remaining = ctx.remaining_accounts();
for acc in remaining.iter() {
    let meta = acc?; // RemainingAccount
}
```

Use when account types vary or you need manual validation. Prefer `parse::<T, N>()` when all trailing accounts share one type.

### Account count trait

`AccountCount` associated constant on Accounts struct defines expected count. Mismatch fails parsing before handler runs.

## Bumps struct

`Ctx<T>` exposes `ctx.bumps: T::Bumps` with fields matching PDA account names:

```rust
pub fn make_escrow(&mut self, receive: u64, bumps: &MakeBumps) -> Result<(), ProgramError> {
    self.escrow.set_inner(EscrowInner {
        /* ... */
        bump: bumps.escrow,
    });
    Ok(())
}
```

Access in handler: `&ctx.bumps` or destructuring.

## Instruction args

Fixed-size args after discriminator are parsed automatically:

```rust
#[instruction(discriminator = 0)]
pub fn transfer(ctx: Ctx<Transfer>, amount: u64, fee_bps: u16) -> Result<(), ProgramError>
```

For complex fixed layouts, use structs implementing `InstructionArg`.

Dynamic args (strings, vecs) require `heap` feature — see [heap-and-dynamic-data.md](heap-and-dynamic-data.md).
