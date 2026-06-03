# CPI and declare_program

**Official:** [Cross-Program Invocations](https://quasar-lang.com/docs/core-concepts/cpi) · [IDL Generation](https://quasar-lang.com/docs/core-concepts/idl)

Cross-program invocation in Quasar uses const-generic builders with stack-allocated account metas. Every CPI returns `CpiCall<'a, ACCTS, DATA>` — chain `.invoke()`, `.invoke_signed(&seeds)`, or `.invoke_with_signers(&[seeds_a, seeds_b])`.

## System program CPI

```rust
#[derive(Accounts)]
pub struct Deposit {
    #[account(mut)]
    pub user: Signer,
    #[account(mut, address = VaultPda::seeds(user.address()))]
    pub vault: UncheckedAccount,
    pub system_program: Program<SystemProgram>,
}

impl Deposit {
    pub fn deposit(&self, amount: u64) -> Result<(), ProgramError> {
        self.system_program
            .transfer(&self.user, &self.vault, amount)
            .invoke()
    }
}
```

`Program<SystemProgram>` implements transfer/create/allocate via `quasar_lang::cpi::system`.

## SPL CPI

See [spl-token.md](spl-token.md). Pattern:

```rust
program_method(args...).invoke()
program_method(args...).invoke_signed(&seeds)?
```

## invoke vs invoke_signed

- **`invoke()`** — no PDA signers; signers must have signed the transaction
- **`invoke_signed(&[&[Seed]])`** — PDA program signers; pass seed slices matching account derivation

## Direct lamport manipulation

When program owns an account (PDA), you may adjust lamports without system CPI:

```rust
use quasar_lang::prelude::*;

let vault = self.vault.to_account_view();
let user = self.user.to_account_view();
set_lamports(vault, vault.lamports() - amount);
set_lamports(user, user.lamports() + amount);
```

**Requirement:** vault owner must be your program ID. See `examples/vault/src/instructions/withdraw.rs`.

## CpiReturn

For CPIs that return data:

```rust
let result: u64 = self.external_program
    .get_value(&account)
    .invoke()?
    .get();
```

## DynCpiCall

When account count is not known at compile time, use dynamic CPI (higher CU). Prefer const-generic `CpiCall` when possible.

## declare_program!

**Derive-supported** (`derive/src/declare_program.rs`) — generates typed CPI helpers from an IDL JSON file at compile time. **No workspace example program uses this yet**; read the derive source and `tests/suite/` CPI tests for integration patterns before adopting.

### Syntax (when IDL is available)

```rust
quasar_lang::declare_program!(vault, "target/idl/vault.json");
```

Path resolves relative to `CARGO_MANIFEST_DIR`.

### Constraints (from derive)

- Only **struct** account types (no enum kinds in CPI size calculation)
- Instruction args must be fixed-size: `u8`–`u128`, `i8`–`i128`, `bool`, `pubkey`/`Address`
- No dynamic/string args in declare_program CPI

### Workflow

1. Build dependency program → produces `target/idl/program.json`
2. `declare_program!(name, "path/to/idl.json")` in dependent crate
3. Add program account to Accounts struct as `Program<GeneratedType>`
4. Verify generated API against `derive/src/declare_program.rs` — do not invent method names

## Remaining accounts in CPI

When target program expects extra accounts, use `CtxWithRemaining` and forward:

```rust
#[instruction(discriminator = 0)]
pub fn proxy(ctx: CtxWithRemaining<Proxy>) -> Result<(), ProgramError> {
    let remaining = ctx.remaining_accounts();
    // pass to DynCpiCall or manual invoke
    Ok(())
}
```

## CPI account ordering

Account metas passed to CPI must match target program's expected order exactly. IDL-generated helpers handle this; manual CPI requires reading target IDL.

## Error handling

CPI failure returns `ProgramError` from callee. Map to custom errors:

```rust
external::call(...).invoke().map_err(|_| ElectionError::CpiFailed)?;
```

Or use `?` to propagate.
