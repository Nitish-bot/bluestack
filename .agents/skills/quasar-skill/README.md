# quasar-skill

Agent skill for authoring **Quasar** zero-copy Solana programs (`quasar-lang`). Covers account macros, constraints, SPL token CPI, events, errors, heap/dynamic data, Anchor migration, and `quasar-svm` testing.

**Not included:** Quasar CLI (`quasar init/build/test/deploy/profile`) — library authoring only.

**Types:** In `quasar_lang::prelude`, `String<N>` and `Vec<T, N>` are Pod aliases (`PodString` / `PodVec`) — not heap `alloc` types. See [references/heap-and-dynamic-data.md](references/heap-and-dynamic-data.md).

Official documentation index: [references/official-docs-index.md](references/official-docs-index.md) — https://quasar-lang.com/docs/

## Quick install

From the `quasar-skill/` directory (where `install.sh` lives):

```bash
./install.sh           # Auto-detect platform, user-level
./install.sh --all     # Install to every detected tool
./install.sh --dry-run # Preview without changes
./install.sh --project # Copy into current repo .agents/skills/
```

## Multi-platform install

### Claude Code (global)

```bash
cp -R quasar-skill ~/.claude/skills/
```

### Cursor (global)

```bash
cp -R quasar-skill ~/.cursor/rules/quasar-skill
./install.sh --platform cursor
```

### Codex CLI / Gemini CLI / universal path

```bash
cp -R quasar-skill ~/.agents/skills/quasar-skill
```

### Project-level

```bash
cp -R quasar-skill .agents/skills/
# or
./install.sh --project
```

## Usage

In any supported agent session:

```
/quasar-skill scaffold a token escrow with PDA vault
Write a Quasar vault program with deposit and withdraw
Migrate this Anchor program to quasar-lang
```

The skill loads glossary-first terminology and lazy-loads detailed references from `references/`.

## Package contents

```
quasar-skill/
├── SKILL.md              # Main skill (invoke with /quasar-skill)
├── references/           # Deep docs + official-docs-index.md
├── assets/               # minimal-counter-template.rs
├── install.sh            # Cross-platform installer
└── README.md
```

## Validation

```bash
python3 ~/.claude/skills/agent-skill-creator/scripts/validate.py quasar-skill/
python3 ~/.claude/skills/agent-skill-creator/scripts/security_scan.py quasar-skill/
```

Both should exit 0 before publishing.

## Source

Derived from the [Quasar](https://github.com/blueshift-gg/quasar) workspace and [official docs](https://quasar-lang.com/docs/). Canonical examples: `examples/escrow`, `examples/vault`, `lang/tests/compile_fail`, `tests/suite`.

## License

MIT — same as Quasar workspace.
