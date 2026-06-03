# Canonical Examples Map

Ground truth: `/home/nitishc/progmag/quasar/examples/` and `tests/programs/*`. Use these before inventing patterns.

## Minimal counter (PDA + set_inner + init)

| File | Pattern |
|------|---------|
| [../assets/minimal-counter-template.rs](../assets/minimal-counter-template.rs) | `address = Counter::seeds(authority.address())`; `bumps.counter` on init |
| `tests/programs/test-misc/src/instructions/initialize.rs` | `init` + `address = SimpleAccount::seeds(payer.address())` |

## examples/vault

| File | Pattern |
|------|---------|
| `examples/vault/src/instructions/deposit.rs` | `#[derive(Seeds)]` + `address = VaultPda::seeds(...)`; `Program<SystemProgram>`; system CPI |
| `examples/vault/src/instructions/withdraw.rs` | Direct `set_lamports` on owned PDA |

## examples/escrow

| File | Pattern |
|------|---------|
| `examples/escrow/src/lib.rs` | `#[instruction(discriminator = 0)]` valid; multi-ix dispatch + events |
| `examples/escrow/src/instructions/make.rs` | PDA init; `Program<TokenProgram>`; token ATAs |
| `examples/escrow/src/instructions/take.rs` | `close(dest = ...)`; PDA-signed SPL CPI |

## examples/multisig

| File | Pattern |
|------|---------|
| `examples/multisig/src/state.rs` | `String<32>`, `Vec<Address, 10>` in account (no heap) |
| `examples/multisig/src/lib.rs` | `CtxWithRemaining`; `remaining_accounts().parse::<Signer, 10>()` |
| `examples/multisig/src/instructions/create.rs` | `Remaining<Signer, 10>` in impl |

## tests/programs (focused)

| Program | Pattern |
|---------|---------|
| `test-errors/` | `#[error_code]` plain variants |
| `test-heap/` | `#[instruction(..., heap)]` vs allocation without heap |
| `test-token-cpi/` | `token_sweep(...)` constraint |

## Anti-patterns (compile-fail)

| File | Pattern |
|------|---------|
| `lang/tests/compile_fail/seeds_raw_on_account.rs` | `seeds = [...]` — forbidden |
| `lang/tests/compile_fail/zero_discriminator_account.rs` | `#[account(discriminator = 0)]` — forbidden |

## Maintainer note

Skill content is validated against repo HEAD (`examples/*`, `tests/programs/*`, `derive/`). Website constraint pages may lag — see [official-docs-index.md](official-docs-index.md).
