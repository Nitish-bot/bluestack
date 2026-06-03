# Testing with quasar-svm

**Official:** [Testing](https://quasar-lang.com/docs/clients/testing)

Program tests use the **`quasar-svm`** crate — an in-process SVM simulator. SPL Token, Token-2022, and ATA programs load by default. This skill does not cover CLI test runners (`quasar test`).

## Dependency

```toml
[dev-dependencies]
quasar-svm = { git = "https://github.com/blueshift-gg/quasar-svm" }
my-program-client = { path = "client" }  # optional instruction builders
```

Host std is allowed in tests only:

```rust
extern crate std;
```

## Basic test harness

From `examples/vault/src/tests.rs`:

```rust
extern crate std;
use {
    quasar_svm::{Account, Instruction, Pubkey, QuasarSvm},
    quasar_vault_client::*,
    std::{println, vec},
};

fn setup() -> QuasarSvm {
    let elf = std::fs::read("../../target/deploy/quasar_vault.so").unwrap();
    QuasarSvm::new().with_program(&crate::ID, &elf)
}

fn signer(address: Pubkey) -> Account {
    quasar_svm::token::create_keyed_system_account(address, 10_000_000_000)
}

#[test]
fn test_deposit() {
    let mut svm = setup();
    let user = Pubkey::new_from_array([1; 32]);
    let (vault, _) = Pubkey::find_program_address(&[b"vault", user.as_ref()], &crate::ID);

    let instruction: Instruction = DepositInstruction {
        user,
        vault,
        system_program: quasar_svm::system_program::ID,
        amount: 1_000_000_000,
    }.into();

    let result = svm.process_instruction(
        &instruction,
        &[signer(user), empty(vault)],
    );

    result.assert_success();
    assert_eq!(result.account(&vault).unwrap().lamports, 1_000_000_000);
}
```

### Multi-step chains

```rust
let deposit_result = svm.process_instruction(&deposit_ix, &accounts);
deposit_result.assert_success();

let withdraw_result = svm.process_instruction(
    &withdraw_ix,
    &deposit_result.accounts,  // post-state from prior ix
);
withdraw_result.assert_success();
```

Use `process_instruction_chain` for sequential instructions in one call. Use `simulate_instruction` for dry runs.

### Builder helpers

```rust
QuasarSvm::new()
    .with_program(&program_id, &elf)
    .with_account(account)
    .with_airdrop(&pubkey, 10_000_000_000)
    .with_slot(100)
    .with_compute_budget(200_000);
```

## Build artifact path

Tests load the compiled `.so`:

```
../../target/deploy/<crate_name_underscored>.so
```

Build the program before tests (via cargo build-sbf or project build tooling). Path is relative to test file location.

## Account fixtures

### System account (signer with lamports)

```rust
quasar_svm::token::create_keyed_system_account(&pubkey, lamports)
```

### Empty account

```rust
Account {
    address: pubkey,
    lamports: 0,
    data: vec![],
    owner: quasar_svm::system_program::ID,
    executable: false,
}
```

### Program-owned PDA with lamports

```rust
Account {
    address: vault_pda,
    lamports: 1_000_000_000,
    data: vec![],
    owner: crate::ID,  // your program
    executable: false,
}
```

### Token accounts

Use helpers from `tests/suite/src/helpers.rs` patterns:

```rust
quasar_svm::token::create_keyed_token_account_with_program(...)
quasar_svm::token::create_keyed_mint_account_with_program(...)
```

## Client instruction builders

Generated or hand-written clients convert to `Instruction`:

```rust
let ix: Instruction = MakeInstruction {
    maker,
    escrow,
    mint_a,
    // ...
    deposit,
    receive,
}.into();
```

See `examples/escrow/client/` for full escrow client.

## Assertions

```rust
// Success
assert!(result.is_ok());

// Error variant
assert_eq!(result.err(), Some(ProgramError::Custom(6000 + code)));

// Account state after ix
let acct = result.account(&pubkey).unwrap();
assert_eq!(acct.lamports, expected);

// CU profiling
println!("CU: {}", result.compute_units_consumed);
```

## Integration test suite

Workspace integration tests in `tests/suite/` cover:
- Init / init(idempotent)
- PDA seeds and bumps
- Token init, transfer, CPI
- Constraints and custom errors
- Discriminator validation
- Realloc, close, sweep
- Remaining accounts
- Heap/no-heap behavior

Run via workspace test configuration — reference these when implementing new constraint types.

## Test module location

**Option A:** `src/tests.rs` with `#[cfg(test)] mod tests;` in lib.rs

**Option B:** `tests/integration.rs` as separate crate

Examples use Option A for co-location with program.

## Pubkey vs Address in tests

Tests use `quasar_svm::Pubkey` (solana SDK type). On-chain code uses `Address`. Convert via `.as_ref()` / `Address::new_from_array(pubkey.to_bytes())` in clients.

## Mollusk alternative

Quasar CLI supports `mollusk` as an alternate test framework. This skill standardizes on **quasar-svm** for examples and generated tests.

## Test checklist

When delivering a program, include tests for:

1. Happy path each instruction
2. Signer missing → error
3. Wrong PDA seeds → error
4. Constraint violation → custom error if defined
5. Token amount conservation (escrow/deposit flows)
6. CU baseline printed or documented for main paths

## No CLI in tests

Do not document or generate `quasar test` invocations. Tests run via:

```bash
cargo test -p my-program
```

Assuming `.so` exists from prior build step.
