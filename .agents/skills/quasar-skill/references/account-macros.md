# Account Macros

**Official:** [Account Layout](https://quasar-lang.com/docs/zero-copy/account-layout) · [Pod Types](https://quasar-lang.com/docs/zero-copy/pod-types) (`#[account]`)

The `#[account]` attribute on a struct generates zero-copy layout, discriminator checks, and trait impls.

## Basic usage

PDA state accounts (canonical minimal program):

```rust
#[account(discriminator = 1, set_inner)]
#[seeds(b"counter", authority: Address)]
pub struct Counter {
    pub authority: Address,
    pub count: u64,
    pub bump: u8,
}
```

Full template: [../assets/minimal-counter-template.rs](../assets/minimal-counter-template.rs).

Generates:
- ZC companion struct with discriminator prefix
- `Owner`, `Discriminator`, `Space` trait implementations (`const SPACE: usize` — discriminator + fixed fields + dynamic headers)

## Discriminator forms

```rust
#[account(discriminator = 1)]           // single byte: [1]
#[account(discriminator = [1, 2, 3])]   // multi-byte
```

**Rules:**
- Must not be all zeros — compile error
- Choose width deliberately; all account discriminators in a program typically share length
- `0xFF` first byte is reserved (conflicts with borrow header sentinel)

## set_inner

For accounts initialized via `#[account(init, ...)]`:

```rust
#[account(discriminator = 1, set_inner)]
#[seeds(b"escrow", maker: Address)]
pub struct Escrow {
    pub maker: Address,
    pub mint_a: Address,
    pub receive: u64,
    pub bump: u8,
}
```

Init handler writes via generated inner type:

```rust
self.escrow.set_inner(EscrowInner {
    maker: *self.maker.address(),
    mint_a: *self.mint_a.address(),
    receive,
    bump: bumps.escrow,
});
```

Without `set_inner`, you assign fields directly on the ZC view (works for simple types after init).

## Typed seeds on account type

```rust
#[account(discriminator = 1)]
#[seeds(b"vault", user: Address)]
pub struct Vault {
    pub bump: u8,
}
```

`#[seeds(...)]` generates `HasSeeds` on the account type (`SEED_PREFIX`, `SEED_DYNAMIC_COUNT`) — see `lang/src/traits.rs`. There is **no** `fn seeds()` on `Vault`.

In `#[derive(Accounts)]`, `address = Vault::seeds(user.address())` is **constraint syntax** parsed by the derive macro (`derive/src/accounts/syntax/attrs.rs`): it wires prefix + args into `AddressVerify` at parse time. It does not return slices or an `Address` at runtime.

For CPI `invoke_signed`, the macro generates a method on the **accounts struct**, e.g. `self.vault_seeds(bumps)` or `self.vault_signer(bumps)`. See `examples/escrow/src/instructions/take.rs`, `examples/multisig/src/instructions/execute_transfer.rs`.

```rust
#[account(init, payer = user, address = Vault::seeds(user.address()))]
pub vault: Account<Vault>,  // space from <Vault as Space>::SPACE — no space = needed
```

**Do not** use `seeds = [...]` or `seeds = Type::seeds(...)` on account fields — compile-fail (`lang/tests/compile_fail/seeds_raw_on_account.rs`). Lamport-only PDAs use `#[derive(Seeds)]` on a helper struct (`examples/vault/`).

## unsafe_no_disc

```rust
#[account(unsafe_no_disc)]
pub struct LegacyAccount { /* ... */ }
```

Skips discriminator prefix. Avoid unless migrating legacy layouts; loses discriminator validation.

## fixed_capacity

```rust
#[account(discriminator = 1, fixed_capacity)]
pub struct Buffer {
    pub items: PodVec<u64, 32>,  // stored inline at full capacity
}
```

All fields treated as fixed-size in ZC struct — no dynamic tail region. Higher rent, simpler access.

## Field type rules

| Field type | Behavior |
|------------|----------|
| `Address`, `u8`–`u128`, `i8`–`i128`, `bool` | Fixed-size in ZC body |
| `PodU64`, etc. | Fixed-size alignment-1 |
| `PodString<N, PFX>` | Dynamic tail; fixed fields must come first |
| `PodVec<T, N>` | Dynamic tail; `T` must be Pod |
| Generic `T` | **Not supported** — compile error |

## Space calculation

`#[account]` implements `Space` with `const SPACE: usize` (discriminator + fixed fields + dynamic region headers). For `#[account(init)]` on `Account<T>`, the derive macro allocates `<T as Space>::SPACE` automatically — **do not** add `space =` unless overriding (see `tests/programs/test-misc` `space_override.rs`).

Canonical init (matches `examples/escrow`, `examples/multisig`):

```rust
#[account(mut, init, payer = maker, address = Escrow::seeds(maker.address()))]
pub escrow: Account<Escrow>,
```

Optional override:

```rust
#[account(mut, init, payer = payer, space = 100, address = MyAccount::seeds(payer.address()))]
pub account: Account<MyAccount>,
```

## Accessing account data

```rust
// Read address of account
let addr = self.escrow.address();  // &Address

// Mutable field access (Pod or native)
self.counter.count += 1;

// PodU64
self.balance.set(self.balance.get().checked_add(amount).ok_or(ProgramError::ArithmeticOverflow)?);
```

## define_account! (advanced)

For manual account types without full derive, see `quasar_lang::macros::define_account!`. Rare — prefer `#[account]`.
