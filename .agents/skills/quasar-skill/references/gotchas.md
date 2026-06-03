# Gotchas and Compile-Fail Patterns

**Official:** [Instructions (discriminators)](https://quasar-lang.com/docs/core-concepts/instructions) · [Accounts and Validation](https://quasar-lang.com/docs/core-concepts/accounts-and-validation)

ADR-style decisions and failure modes from `lang/tests/compile_fail/` and production examples.

## Discriminator rules

Per official docs: all instruction discriminators must share the same byte length; no duplicates; no `0xFF` prefix (reserved for event self-CPI). **Account** discriminators must be non-zero and cannot start with `0xFF`.

### Account: all-zero banned

```rust
// COMPILE FAIL — derive/src/helpers.rs validate_discriminator_not_zero
#[account(discriminator = 0)]
#[account(discriminator = [0])]
```

Error: `discriminator must contain at least one non-zero byte`

**Why:** Zero discriminator indistinguishable from uninitialized account data.

### Instruction: single-byte zero is valid

```rust
// VALID — examples/escrow/src/lib.rs, tests/programs/test-errors/
#[instruction(discriminator = 0)]
pub fn make(ctx: Ctx<Make>) -> Result<(), ProgramError> { ... }
```

Single-byte `discriminator = 0` is allowed. The dispatch path rejects empty instruction data separately (`derive/src/instruction.rs`).

### Instruction: multi-byte all-zero banned

```rust
// COMPILE FAIL — multi-byte only (derive/src/instruction.rs)
#[instruction(discriminator = [0, 0])]
```

Error: `instruction discriminator must contain at least one non-zero byte`

Multi-byte all-zero discriminators are rejected because zeroed instruction data could accidentally match.

### Uniform length across instructions

All `#[instruction(discriminator = ...)]` in a program must use the same byte width. Mixing `discriminator = 1` and `discriminator = [1,2]` fails.

### Uniqueness

Duplicate instruction discriminators fail at compile time.

## Account layout

### Generic account fields

```rust
// COMPILE FAIL
#[account(discriminator = 1)]
pub struct Bad<T> { pub field: T }
```

Account types cannot be generic.

### Wrong account field types in `#[derive(Accounts)]`

```rust
// WRONG — Anchor lifetimes / Rust mut references (not used in repo examples)
pub counter: Account<'info, Counter>,
pub counter: &'info mut Account<Counter>,
pub authority: &'info Signer,

// CORRECT — mutability via #[account(mut)], owned wrapper types
#[account(mut, has_one(authority))]
pub counter: Account<Counter>,
pub authority: Signer,
```

See `examples/vault/src/instructions/deposit.rs`, `examples/escrow/src/instructions/make.rs`.

### Fixed before dynamic

```rust
// COMPILE FAIL — lang/tests/compile_fail/ (Pod field ordering)
pub struct Bad {
    pub name: PodString<32, 2>,
    pub id: u64,
}
```

Fixed fields must precede all `PodString`/`PodVec` fields.

## PodString / PodVec

### Invalid PFX

```rust
// COMPILE FAIL — pod_string_invalid_pfx
const _: () = PodString::<10, 3>::VALID;
```

PFX must be 1, 2, 4, or 8.

### N exceeds PFX capacity

```rust
// COMPILE FAIL — pod_string_n_exceeds_pfx, pod_vec_n_exceeds_pfx
PodString<300, 1>  // length prefix can't represent 300 with 1-byte prefix
```

Maximum storable length depends on PFX width.

## Seeds / PDA address

### Use `address = Type::seeds(...)` — not `seeds =` or bare `bump`

**Wrong — removed key-value directives (compile-fail):**

```rust
#[account(init, payer = creator, seeds = MultisigConfig::seeds(creator), bump)]
#[account(seeds = [b"vault", config], bump)]
```

Error: `unknown key-value directive 'seeds = ...'`

**Correct — typed PDA via `address` (multisig, escrow, vault, test-misc, test-pda):**

```rust
// examples/multisig/src/instructions/create.rs
#[account(init, payer = creator, address = MultisigConfig::seeds(creator.address()))]
pub config: Account<MultisigConfig>,
```

`address = Type::seeds(arg.address())` invokes `AddressVerify` at parse time (`derive/src/accounts/emit/parse.rs`). Fixed pubkeys use `address = CONST` without `::seeds` (`tests/programs/test-errors/src/instructions/address_default.rs`).

### Lamport PDAs without state account

Use `#[derive(Seeds)]` on a helper struct (`examples/vault/src/instructions/deposit.rs`, `examples/multisig/src/instructions/deposit.rs`):

```rust
#[derive(Seeds)]
#[seeds(b"vault", config: Address)]
pub struct MultisigVaultPda;

#[account(mut, address = MultisigVaultPda::seeds(config.address()))]
pub vault: UncheckedAccount,
```

### Wrong arity

```rust
#[account(discriminator = 1)]
#[seeds(b"vault", user: Address)]
pub struct Vault {}

// COMPILE FAIL — wrong arg count to ::seeds()
#[account(address = Vault::seeds(user.address(), extra_arg))]
```

### `Type::seeds(...)` in `invoke_signed`

`address = Counter::seeds(authority.address())` is derive constraint syntax — not a runtime seed array. Use `self.counter_seeds(&ctx.bumps)`, `self.vault_signer(bumps)`, or manual `Seed` slices. Wrong: `.invoke_signed(&[Escrow::seeds(maker.address())])`.

## Token program selectors

```
multi_token_program_missing_mint_selector
multi_token_program_missing_token_selector
```

When using interface token types with multiple token programs, include required program selector accounts. Read the compile error — it names the missing account.

## Events

| Test file | Issue |
|-----------|-------|
| `event_unsupported_type` | Field type not supported |
| `event_unsupported_string` | String fields not allowed |
| `event_tuple_struct` | Tuple structs not supported |

Use named struct fields with Pod-compatible types.

## Realloc

| Test file | Issue |
|-----------|-------|
| `realloc_bad_signer` | Payer not signer |
| `realloc_bad_optional` | Realloc on optional account |
| `realloc_bad_unchecked` | Realloc on UncheckedAccount incorrectly |

## Close

| Test file | Issue |
|-----------|-------|
| `close_mint` | Cannot close mint accounts via close constraint |

## Error codes

| Test file | Issue |
|-----------|-------|
| `error_code_on_struct` | `#[error_code]` only on enums |

## Metadata

| Test file | Issue |
|-----------|-------|
| `metadata_bad_rent` | Invalid rent/sysvar setup for metadata init |

## Runtime gotchas

### PodU64 wrapping

Release build: `balance + amount` wraps on overflow. **Always** use `checked_add` for financial logic.

### Bump not stored

Re-deriving PDA bump every instruction wastes CU. Store `bump: u8` in account on init.

### Token authority mismatch

Vault TA owned by escrow PDA but transfer uses maker as signer → `InsufficientFunds` / signature error. Use `invoke_signed`.

### Withdraw on non-owned PDA

Direct lamport manipulation requires program ownership. System-owned PDAs need system CPI.

### Heap in wrong instruction

Allocating in instruction without `heap` attribute → abort in release. Add `#[instruction(..., heap)]` and `alloc` feature.

### 0xFF borrow state

Framework internal — do not use 0xFF as first discriminator byte.

## Scenario stress tests

**Q: Can two instructions share discriminator if accounts differ?**
A: No. Discriminator is the only routing key in dispatch.

**Q: Can account discriminator match instruction discriminator?**
A: Technically yes (different namespaces) but avoid for human clarity.

**Q: init(idempotent) on PDA — who pays if account exists?**
A: Rent only charged on creation; existing path validates constraints only.

**Q: Empty seed slice PDA?**
A: Supported — see `test-pda/init_empty_seed`. Still need bump handling.

## Miri / unsafe

Quasar uses unsafe pointer casts. Run `cargo miri test` in `lang/` when modifying ZC layouts. Alignment-1 is mandatory for Pod types.
