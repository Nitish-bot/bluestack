# Official Quasar Documentation Index

Canonical docs: **https://quasar-lang.com/docs/** — machine-readable TOC: https://quasar-lang.com/llms.txt

> **Stale constraint pages:** Reference pages under [Account Constraints](https://quasar-lang.com/docs/references/account-constraints) and similar lookup tables may lag **repo HEAD**. For `#[account(...)]` attributes and PDA syntax, prefer **`examples/escrow`**, **`examples/vault`**, **`examples/multisig`**, and **`tests/programs/*`** over website-only examples.

This skill focuses on **library authoring** (`quasar-lang`, `quasar-spl`, `quasar-svm`). Skip CLI install/deploy pages unless the user asks about toolchain.

## Getting Started

| Topic | URL |
|-------|-----|
| Installation | https://quasar-lang.com/docs/getting-started/installation |
| Quickstart | https://quasar-lang.com/docs/getting-started/quickstart |
| Migrating from Anchor | https://quasar-lang.com/docs/getting-started/migrating-from-anchor |

## Core Concepts

| Topic | URL |
|-------|-----|
| Program Structure | https://quasar-lang.com/docs/core-concepts/program-structure |
| Accounts and Validation | https://quasar-lang.com/docs/core-concepts/accounts-and-validation |
| Instructions | https://quasar-lang.com/docs/core-concepts/instructions |
| Program Derived Addresses | https://quasar-lang.com/docs/core-concepts/pda |
| Cross-Program Invocations | https://quasar-lang.com/docs/core-concepts/cpi |
| IDL Generation | https://quasar-lang.com/docs/core-concepts/idl |

## SPL Tokens

| Topic | URL |
|-------|-----|
| Token Program | https://quasar-lang.com/docs/spl-tokens/token-program |
| Token-2022 and Interfaces | https://quasar-lang.com/docs/spl-tokens/token-2022 |
| Associated Token Accounts | https://quasar-lang.com/docs/spl-tokens/associated-token-accounts |

## Clients and Testing

| Topic | URL |
|-------|-----|
| TypeScript Client | https://quasar-lang.com/docs/clients/typescript |
| Rust Client | https://quasar-lang.com/docs/clients/rust |
| Testing (QuasarSVM, Mollusk) | https://quasar-lang.com/docs/clients/testing |

## Zero-Copy Deep Dive

| Topic | URL |
|-------|-----|
| Pod Types | https://quasar-lang.com/docs/zero-copy/pod-types |
| Account Layout | https://quasar-lang.com/docs/zero-copy/account-layout |
| Dynamic Fields | https://quasar-lang.com/docs/zero-copy/dynamic-fields |

## Guides (canonical program patterns)

| Topic | URL |
|-------|-----|
| Build a Vault | https://quasar-lang.com/docs/guides/build-a-vault |
| Build an Escrow | https://quasar-lang.com/docs/guides/build-an-escrow |
| Build a Multisig | https://quasar-lang.com/docs/guides/build-a-multisig |

## Reference (lookup tables)

| Topic | URL |
|-------|-----|
| Account Types | https://quasar-lang.com/docs/references/account-types |
| Account Constraints | https://quasar-lang.com/docs/references/account-constraints |
| Type Mapping | https://quasar-lang.com/docs/references/type-mapping |
| Configuration | https://quasar-lang.com/docs/references/configuration |

## Profiling (out of skill scope unless user asks)

| Topic | URL |
|-------|-----|
| CU Profiler | https://quasar-lang.com/docs/profiling/cu-profiler |
| Flamegraphs | https://quasar-lang.com/docs/profiling/flamegraphs |
| Benchmarks | https://quasar-lang.com/docs/profiling/benchmarks |

## Skill reference mapping

| Skill file | Primary official pages |
|------------|------------------------|
| [glossary.md](glossary.md) | Accounts and Validation, Pod Types |
| [program-anatomy.md](program-anatomy.md) | Program Structure |
| [account-macros.md](account-macros.md) | Account Layout, Pod Types |
| [accounts-constraints.md](accounts-constraints.md) | Accounts and Validation, Account Constraints |
| [spl-token.md](spl-token.md) | Token Program, Token-2022, ATA |
| [cpi-and-declare-program.md](cpi-and-declare-program.md) | CPI, IDL |
| [heap-and-dynamic-data.md](heap-and-dynamic-data.md) | Dynamic Fields |
| [anchor-migration.md](anchor-migration.md) | Migrating from Anchor |
| [testing.md](testing.md) | Testing |
| [errors-events-remaining.md](errors-events-remaining.md) | CPI (events), Instructions (return data) |
