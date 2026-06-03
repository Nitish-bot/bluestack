---
name: quasar-skill
description: >-
  Author zero-copy, no_std Solana programs with Quasar (quasar-lang). Covers
  #[program], #[account], #[derive(Accounts)], PodU64, explicit discriminators,
  Ctx, SPL token CPI, declare_program CPI, heap/dynamic data, events, errors,
  and Anchor migration. Activates on quasar, quasar-lang, zero-copy solana,
  no_std solana, #[program] quasar, PodU64, migrate anchor quasar, quasar spl.
license: MIT
metadata:
  author: quasar-lang
  version: 1.0.0
  created: 2026-06-01
  last_reviewed: 2026-06-02
  review_interval_days: 90
activation: /quasar-skill
provenance:
  maintainer: quasar-lang contributors
  version: 1.0.0
  created: 2026-06-01
  source_references:
    - https://github.com/blueshift-gg/quasar
    - https://quasar-lang.com/docs/
    - https://quasar-lang.com/llms.txt
compatibility: >-
  Works on all platforms supporting the Agent Skills Open Standard (SKILL.md):
  Claude Code, Cursor, Codex CLI, Gemini CLI, and 20+ others.
---
# /quasar-skill

You are a Quasar Solana program author: **zero-copy, `no_std`**, explicit discriminators, no Borsh by default. Stack: `quasar-lang`, `quasar-derive`, `quasar-pod`, `quasar-spl`.

**Scope:** on-chain library crates and `quasar-svm` tests. **Out of scope:** CLI (`quasar init/build/test/deploy`).

## Terminology (brief)

Full glossary: [references/glossary.md](references/glossary.md).

| Term | Quasar meaning |
|------|----------------|
| **Account\<T\>** | ZC view over `#[repr(C)]` data — not Borsh |
| **Address** | 32-byte pubkey (`solana_address::Address`) |
| **Ctx\<T\>** | Parsed accounts + bumps |
| **discriminator** | Explicit bytes on `#[account]` and `#[instruction]` |
| **String\<N\> / Vec\<T,N\>** | Prelude aliases for `PodString` / `PodVec` — not heap types |
| **PodU64** | `[u8;8]` integer in account bodies |

## Activation

`/quasar-skill` or natural language: scaffold escrow, migrate Anchor, SPL CPI, multisig.

**Do not activate** for Anchor-only work, pure client SDK, or CLI-only questions.

## Workflow

1. **Instruction surface** — List each ix with `#[instruction(discriminator = N)]` (uniform width, unique). Account structs + constraints. Args: fixed or Pod `String`/`Vec` unless handler allocates (then `heap`).

2. **Scaffold** — `#![no_std]`, `#[program]`, `declare_id!`, `mod instructions/state`. `crate-type = ["cdylib","lib"]`. Template: [assets/minimal-counter-template.rs](assets/minimal-counter-template.rs). Layout: [references/program-anatomy.md](references/program-anatomy.md).

3. **Account types** — `#[account(discriminator = N, set_inner)]`, `#[seeds(...)]` on type, store `bump: u8`. Account discriminators **must be non-zero**. Macros: [references/account-macros.md](references/account-macros.md).

4. **Accounts structs** — `#[derive(Accounts)]`; `#[account(mut)]` on fields; PDA init: `address = Type::seeds(arg.address())` — not `seeds =` / bare `bump`. Constraints: [references/accounts-constraints.md](references/accounts-constraints.md). Patterns: `examples/escrow/`, `examples/vault/`, `examples/multisig/`.

5. **Handlers** — `Ctx<T>` or `CtxWithRemaining<T>`; thin `#[program]` dispatch; logic in `impl AccountsStruct`. Errors/events/remaining: [references/errors-events-remaining.md](references/errors-events-remaining.md).

6. **SPL** — `quasar_spl`, `Program<TokenProgram>`, `TokenCpi` methods. [references/spl-token.md](references/spl-token.md) · `examples/escrow/`.

7. **CPI** — `.invoke()` / `.invoke_signed(&seeds)`. System: `Program<SystemProgram>`. External IDL: [references/cpi-and-declare-program.md](references/cpi-and-declare-program.md).

8. **Dynamic data** — Pod `String`/`Vec` in state/ix without heap; handler allocation needs `alloc` + `#[instruction(..., heap)]`. [references/heap-and-dynamic-data.md](references/heap-and-dynamic-data.md).

9. **Test** — `quasar-svm`, ELF from `target/deploy/`. [references/testing.md](references/testing.md).

## Anchor migration

Porting Anchor only: [references/anchor-migration.md](references/anchor-migration.md). Greenfield: skip it.

## Gotchas

Read [references/gotchas.md](references/gotchas.md) before debugging.

| Symptom | Cause | Fix |
|---------|-------|-----|
| `discriminator must contain at least one non-zero byte` | `#[account(discriminator = 0)` | Non-zero account discriminator |
| `instruction discriminator must contain...` | Multi-byte ix discriminator all zeros | Use non-zero bytes in `[u8,...]` form |
| `unknown key-value directive 'seeds'` | `seeds =` on field | `address = Type::seeds(...)` |
| `fixed fields must precede PodString/PodVec` | Field order | Reorder account |
| Silent balance wrap | `PodU64` `+` in release | `checked_add` |
| Alloc abort | Heap alloc without `heap` ix | `#[instruction(..., heap)]` + `alloc` feature |

**Valid:** `#[instruction(discriminator = 0)]` single-byte (`examples/escrow`, `test-errors`).

## Canonical repo examples

Map: [references/canonical-examples.md](references/canonical-examples.md). Prefer repo over stale website constraint pages — [references/official-docs-index.md](references/official-docs-index.md).

| Example | Path |
|---------|------|
| Counter template | [assets/minimal-counter-template.rs](assets/minimal-counter-template.rs) |
| Token escrow + events | `examples/escrow/` |
| Lamport vault + system CPI | `examples/vault/` |
| Multisig + remaining accounts | `examples/multisig/` |
| Compile-fail patterns | `lang/tests/compile_fail/` |
| Integration tests | `tests/suite/`, `tests/programs/*` |

## Reference index

| Topic | File |
|-------|------|
| Official docs (stale warning) | [references/official-docs-index.md](references/official-docs-index.md) |
| Glossary | [references/glossary.md](references/glossary.md) |
| Program anatomy | [references/program-anatomy.md](references/program-anatomy.md) |
| `#[account]` macro | [references/account-macros.md](references/account-macros.md) |
| Constraints | [references/accounts-constraints.md](references/accounts-constraints.md) |
| SPL | [references/spl-token.md](references/spl-token.md) |
| CPI / declare_program | [references/cpi-and-declare-program.md](references/cpi-and-declare-program.md) |
| Errors, events, remaining | [references/errors-events-remaining.md](references/errors-events-remaining.md) |
| Heap & Pod types | [references/heap-and-dynamic-data.md](references/heap-and-dynamic-data.md) |
| Anchor migration | [references/anchor-migration.md](references/anchor-migration.md) |
| Gotchas | [references/gotchas.md](references/gotchas.md) |
| Testing | [references/testing.md](references/testing.md) |

## Output standards

1. Explicit discriminators on every account type and instruction
2. `cdylib` + correct features in `Cargo.toml`
3. PDA bumps stored when reused
4. `checked_*` on financial Pod math
5. `quasar-svm` tests with realistic fixtures
6. No CLI steps unless user asks

Toolchain docs: https://quasar-lang.com/docs/getting-started/installation
