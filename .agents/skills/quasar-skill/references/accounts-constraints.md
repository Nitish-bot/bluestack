# Account Constraints

**Official:** [Accounts and Validation](https://quasar-lang.com/docs/core-concepts/accounts-and-validation) · [Account Constraints Reference](https://quasar-lang.com/docs/references/account-constraints) (`#[account(...)]`)

Complete reference for field attributes on `#[derive(Accounts)]` structs. Parsed from `derive/src/accounts/syntax/attrs.rs`.

## Signer and mutability

```rust
#[account(mut)]           // writable
pub user: Signer,

#[account(signer)]        // explicit signer (Signer type implies this)
pub admin: Signer,
```

## Initialization

```rust
#[account(init, payer = payer)]
pub account: Account<MyAccount>,

#[account(init(idempotent), payer = payer)]
pub maybe: Account<MyAccount>,
```

For `Account<T>`, allocation size defaults to `<T as Space>::SPACE` from the `#[account]` macro. Examples: `examples/escrow/src/instructions/make.rs`, `examples/multisig/src/instructions/create.rs`.

| Attribute | Purpose |
|-----------|---------|
| `init` | Create new account; must be empty |
| `init(idempotent)` | Create or validate existing (replaces Anchor `init_if_needed`) |
| `payer = NAME` | Signer funding rent (required for init) |
| `space = EXPR` | **Optional** override of `<T as Space>::SPACE` (rare; see `test-misc` `space_override.rs`) |

Token/mint/ATA init uses behavior groups — also no `space =`:

```rust
#[account(init(idempotent), payer = maker, token(mint = mint_a, authority = escrow, token_program = token_program))]
pub vault_ta: Account<Token>,
```

## PDA constraints

**Use `address = Type::seeds(args)` — not `seeds =` or bare `bump`.** Typed PDA init/validate passes seed args to `AddressVerify` via the `address` directive. Canonical multisig init (`examples/multisig/src/instructions/create.rs` lines 10–11):

```rust
#[account(init, payer = creator, address = MultisigConfig::seeds(creator.address()))]
pub config: Account<MultisigConfig>,
```

Spelling is `MultisigConfig` (not `MultiSigConfig`). Type name + seed args must match `#[seeds(b"multisig", creator: Address)]` on the account struct (`examples/multisig/src/state.rs`).

```rust
// Validate existing PDA (escrow take/refund)
#[account(
    mut,
    has_one(maker),
    address = Escrow::seeds(maker.address())
)]
pub escrow: Account<Escrow>,

// Init with PDA (no space = — auto from Space trait)
#[account(mut, init, payer = maker, address = Escrow::seeds(maker.address()))]
pub escrow: Account<Escrow>,

// Lamport vault — separate #[derive(Seeds)] type (examples/vault, multisig deposit)
#[derive(Seeds)]
#[seeds(b"vault", user: Address)]
pub struct VaultPda;

#[account(mut, address = VaultPda::seeds(user.address()))]
pub vault: UncheckedAccount,
```

Fixed pubkey (no PDA derivation) — `tests/programs/test-errors/src/instructions/address_default.rs`:

```rust
#[account(address = EXPECTED_ADDR_DEFAULT)]
pub admin: UncheckedAccount,
```

### What `Type::seeds(args)` is (and is not)

| Layer | What it is | Returns / produces |
|-------|------------|-------------------|
| `#[seeds(b"prefix", arg: Ty)]` on account struct | Seed schema on state type | `HasSeeds` metadata only |
| `#[derive(Seeds)]` + `#[seeds(...)]` on helper struct | Lamport PDA without `Account<T>` state | `VaultPda::seeds(user.address())` for `address =` |
| `address = Counter::seeds(authority.address())` on field | Derive constraint syntax | PDA verify at parse; **not** a Rust method on `Counter` |
| `self.counter_seeds(&InitializeBumps)` on **Accounts** struct | Generated CPI helper | `[quasar_lang::cpi::Seed; N]` for `invoke_signed` |

`Counter::seeds(authority.address())` does **not** return seed slices you can pass to CPI. Use `self.escrow_seeds(bumps)`, `self.vault_signer(bumps)`, or manual `Seed` slices (`examples/escrow/src/instructions/refund.rs`).

### Removed: `seeds =` and `bump` key-value directives

```rust
// COMPILE FAIL — lang/tests/compile_fail/seeds_raw_on_account.rs
#[account(seeds = [b"config"], bump = config.bump)]
#[account(seeds = Vault::seeds(), bump = vault.bump)]
```

Error: `unknown key-value directive 'seeds = ...'`. Use `address = Type::seeds(...)` instead.

### Where `bump` comes from

| Mechanism | Role |
|-----------|------|
| Parse-time PDA verify via `address = Type::seeds(...)` | Derive finds canonical bump; exposes `u8` on `{Accounts}Bumps` (e.g. `bumps.escrow`, `bumps.config`) |
| `pub bump: u8` in account struct | Persist bump on init (`bumps.counter` → `set_inner`) for documentation and manual CPI |
| `ctx.bumps.counter` | Handler access to parse-time bump (`Ctx<T>`) |

Store bump in account state when you build CPI seed arrays manually or want stable documentation (`examples/escrow`, `examples/multisig`).

## Ownership and address

```rust
#[account(mut, has_one(authority), address = Counter::seeds(authority.address()))]
pub counter: Account<Counter>,

#[account(has_one(owner) @ MyError::WrongOwner)]
pub vault: Account<Vault>,

#[account(address = ADMIN_PUBKEY)]
pub admin: Signer,

#[account(address = config.admin @ MyError::NotAdmin)]
pub admin: UncheckedAccount,
```

| Attribute | Purpose |
|-----------|---------|
| `has_one(field)` | Account's `field` must equal this account's address |
| `has_one(field) @ Error` | Custom error on mismatch |
| `address = EXPR` | Account address must match expression (PDA via `Type::seeds(...)`, or fixed pubkey) |
| `constraints(EXPR)` | Arbitrary bool expression |
| `constraints(EXPR) @ Error` | With custom error |

## Close

```rust
#[account(mut, close(dest = receiver))]
pub account: Account<MyAccount>,
```

| Attribute | Purpose |
|-----------|---------|
| `close(dest = NAME)` | Close account; lamports to `NAME` |

## Token sweep (SPL)

Sweep remaining SPL token balance at end of instruction (not lamport rent sweep):

```rust
#[account(
    mut,
    token(mint = mint, authority = authority, token_program = token_program),
    token_sweep(receiver = receiver, mint = mint, authority = authority, token_program = token_program),
)]
pub source: Account<Token>,
```

Canonical: `tests/programs/test-token-cpi/src/instructions/sweep_token.rs`

## Reallocation

Requires `alloc` feature:

```rust
#[account(mut, realloc = new_size, realloc(payer = payer))]
pub account: Account<DynamicAccount>,
```

| Attribute | Purpose |
|-----------|---------|
| `realloc = EXPR` | New account data size |
| `realloc(payer = NAME)` | Rent payer for size increase |

## SPL Token constraints

```rust
#[account(
    mut,
    init(idempotent),
    payer = maker,
    token(mint = mint_a, authority = escrow, token_program = token_program),
)]
pub vault_ta: Account<Token>,
```

| Behavior group | Purpose |
|----------------|---------|
| `token(mint = NAME, authority = NAME, token_program = NAME)` | Token account init/validate |
| `mint(decimals = EXPR, authority = NAME, ...)` | Mint init |
| `associated_token(mint = NAME, authority = NAME, ...)` | ATA init |

## Metaplex metadata (quasar-spl)

```rust
#[account(
    init,
    payer = payer,
    metadata(name = name, symbol = symbol, uri = uri, seller_fee_basis_points = fee, is_mutable = true),
)]
pub metadata: UncheckedAccount,
```

## Master edition

```rust
#[account(master_edition(max_supply = 0))]
pub edition: UncheckedAccount,
```

## dup

```rust
#[account(dup)]
pub same_account_twice: Account<MyType>,
```

Allows the same account pubkey to appear multiple times in the account list (normally forbidden).

## Custom errors on constraints

Append `@ ErrorVariant` or `@ 6000 + MyError::Variant`:

```rust
#[account(constraints(amount > 0) @ MyError::ZeroAmount)]
```

## Multi-token-program selector

When using `InterfaceAccount<Token>` with both SPL Token and Token-2022, additional selector accounts may be required. Missing selector causes compile errors — see `lang/tests/compile_fail/multi_token_program_*`.

## Constraint evaluation order

Init → address verify → owner/type checks → has_one → constraints. Failed checks return `ProgramError` or custom error if specified.
