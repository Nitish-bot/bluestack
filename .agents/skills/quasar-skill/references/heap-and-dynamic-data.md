# Heap and Dynamic Data

**Official:** [Dynamic Fields](https://quasar-lang.com/docs/zero-copy/dynamic-fields) · [Instructions (dynamic args)](https://quasar-lang.com/docs/core-concepts/instructions)

Quasar defaults to **zero heap allocation** in the entrypoint.

## Prelude aliases: `String` / `Vec`

In `quasar_lang::prelude`, `String<N>` and `Vec<T, N>` are **Pod aliases** (`lang/src/lib.rs`):

```rust
pub use crate::pod::{PodString as String, PodVec as Vec};
```

They are **not** Rust `alloc::String` / `Vec`. Use them in account bodies and fixed-layout instruction args without the heap feature.

**Example without heap:** `examples/multisig/src/state.rs` — `label: String<32>`, `signers: Vec<Address, 10>`; handler `set_label(ctx, label: String<32>)` (`examples/multisig/src/lib.rs`).

## When you need heap

- Handler code that **allocates** (e.g. building a runtime `Vec` in the handler body)
- `realloc` account attribute
- Some paths in programs with `any_heap` (event dispatch)

Enable:

1. `alloc` feature on the program crate
2. `#[instruction(discriminator = N, heap)]` on instructions that allocate

Canonical matrix: `tests/programs/test-heap/` (`heap_vec_ok` vs `no_heap_alloc_attempt`).

## PodString / PodVec (explicit names)

Same layout as prelude `String`/`Vec`; use explicit names when documenting:

```rust
pub label: PodString<32, 2>,  // max 32 bytes, 2-byte length prefix
pub items: PodVec<u64, 16>,
```

**PFX rules:** prefix width must be 1, 2, 4, or 8 (compile-time checked).

**Field ordering:** all fixed fields before any `PodString`/`PodVec` — see [gotchas.md](gotchas.md).

## fixed_capacity alternative

Avoid heap entirely by inlining full capacity:

```rust
#[account(discriminator = 1, fixed_capacity)]
pub struct Buffer {
    pub items: PodVec<u64, 32>,
}
```

Higher rent, simpler CU, no `alloc` feature for access.

## PodU64 arithmetic

```rust
self.balance.set(
    self.balance.get()
        .checked_add(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?
);
```

Release builds: `+` on Pod types **wraps silently**.

## Instruction args

| Kind | Heap needed? | Example |
|------|----------------|---------|
| Fixed (`u64`, `Address`) | No | `examples/escrow` |
| Pod `String<N>` / `Vec<T, N>` in ix data | No | `examples/multisig` `set_label` |
| Handler allocates | Yes — `#[instruction(..., heap)]` + `alloc` | `tests/programs/test-heap/` |

## Realloc

```rust
#[account(
    mut,
    realloc = <Profile as Space>::SPACE + extra,
    realloc::payer = payer,
)]
pub profile: Account<Profile>,
```

Requires `alloc` feature. Compile failures: `lang/tests/compile_fail/realloc_*`

## no_heap_alloc_attempt behavior

From `test-heap`:

- **Release (no debug):** heap cursor past end → alloc returns null → abort
- **Debug feature:** heap initialized → alloc succeeds (for testing)

## Debug feature

```toml
[features]
debug = []
```

Pod arithmetic panics on overflow instead of wrapping. Use in dev/test builds, not mainnet.
