# P-Token Formal Verification Guide

## Overview

This guide explains how to run formal verification for the p-token Solana program using the runtime-verification feature and cheatcode functions.

## Architecture

After the merge of `dc/test-hack` branch, the codebase uses conditional compilation to separate production and verification code:

- **Production code**: `src/entrypoint.rs` - Used for normal builds
- **Verification code**: `src/entrypoint-runtime-verification.rs` - Used when `runtime-verification` feature is enabled

## Cheatcode Functions

Cheatcode functions are markers used by the formal verification tools to inject assumptions about account types:

```rust
fn cheatcode_is_account(_: &AccountInfo) {}
fn cheatcode_is_mint(_: &AccountInfo) {}
fn cheatcode_is_rent(_: &AccountInfo) {}
fn cheatcode_is_multisig(_: &AccountInfo) {} // Currently unsupported and behind feature flag "multisig"
```

These functions are no-ops at runtime but provide type hints to the verification tools.

## Running Verification

### Prerequisites

1. Ensure submodules are initialized:
   ```bash
   cd test-properties
   ./setup.sh
   ```

2. Install `uv` if not already installed (Python package manager)

### Running Tests

#### Run specific test:
```bash
cd test-properties
./run-verification.sh test_process_transfer
```

#### Run all tests:
```bash
cd test-properties
./run-verification.sh -a
```

#### Custom options:
```bash
# With custom timeout (in seconds)
./run-verification.sh -t 600 test_process_transfer

# With custom prove-rs options
./run-verification.sh -o "--max-iterations 50 --max-depth 300" test_process_transfer

# With multisig feature enabled
./run-verification.sh --multisig test_process_transfer
```

## Test Functions

All test functions are located in `src/entrypoint-runtime-verification.rs` and follow the pattern:
- `test_process_*` functions for testing individual instructions
- Each function has cheatcode calls at the beginning to mark account types
- Functions use fixed-size arrays for formal verification compatibility

## Feature Flags

### runtime-verification
Required for all verification tests. Enables the verification-specific entrypoint (entrypoint-runtime-verification.rs) and test functions.

### multisig (optional)
Enables cheat codes for all Owner / Authority accounts to be `Multisig` (by default these are `Account`). When enabled:
- Owner / Authority accounts are instantiated as `Multisig` with symbolic arguments via `cheatcode_is_multisig` (TODO: not implemented yet)
- `Multisig`-specific validation logic is included in proof harnesses

Use `--multisig` flag with run-verification.sh to enable this feature.

## Available Tests

See `tests.md` for the complete list of available test functions and their current status.

## Troubleshooting

### Linker Error (_sol_memcpy_)
This is a known issue with the current setup and doesn't affect the verification process. The verification tools work with the SMIR representation, not the linked binary.

### Module Not Found
If you get errors about the entrypoint module not being found, ensure you're building with the `runtime-verification` feature:
```bash
cargo build --features runtime-verification
```

### Multisig Support
To enable multisig account validation in verification tests, use both features:
```bash
cargo build --features "runtime-verification,multisig"
```

## Notes

- The verification process can take significant time (20+ minutes per test)
- Default settings: max-depth 200, max-iterations 30, timeout 1200s
- Results are stored in `artefacts/proof/` directory