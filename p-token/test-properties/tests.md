Proofs timings and steps with default settings from run-proofs.sh:
max-depth 200, max-iterations 30, timeout 20min (1200)

|                                                     |      |         | Steps |                                                                             |
| Start symbol name                                   | Sec  | Status  | max   |                                                                             |
|-----------------------------------------------------|------|---------|-------|-----------------------------------------------------------------------------|
| entrypoint::test_process_transfer                   | 1200 | Timeout | 2178  | erratic branching on invalid IAcc projection, reads alloc (err case)        |
| entrypoint::test_process_mint_to                    | 340  | Stuck   | 1136  | inconsistent projection involving IAcc                                      |
| entrypoint::test_process_burn                       | 692  | Stuck   | 1684  | call to raw_eq (with alloc as arg.), reads alloc (err case)                 |
| entrypoint::test_process_close_account              | 520  | Stuck   | 1468  | call to raw_eq (with alloc as arg.), reads alloc                            |
| entrypoint::test_process_transfer_checked           | 1080 | Stuck   | 1951  | ptr offset, reads alloc (err case)                                          |
| entrypoint::test_process_burn_checked               | 670  | Stuck   | 1684  | call to raw_eq (with alloc as arg.), reads alloc (err case)                 |
| entrypoint::test_process_initialize_account3        | 700  | Stuck   | 610   | branching/stuck on thunked ptr cast, "ExposeAddress", assert_inhab, vacuous |
| entrypoint::test_process_initialize_mint2_freeze    | 412  | Stuck   | 275   | branching/stuck on thunked ptr cast, "ExposeAddress", assert_inhab, vacuous |
| entrypoint::test_process_initialize_mint2_no_freeze | 390  | Stuck   | 275   | branching/stuck on thunked ptr cast, "ExposeAddress", assert_inhab, vacuous |
| entrypoint::test_process_revoke                     | 1200 | Timeout | 955   | non-det branch on cheatcodes (invalid read from stack), reads alloc         |
| entrypoint::test_process_freeze_account             | 1200 | Timeout | 1903  | various branches on key prefix values, reads alloc (err case)               |
| entrypoint::test_process_thaw_account               | 1200 | Timeout | 1903  | various branches on key prefix values, reads alloc (err case)               |
| entrypoint::test_process_mint_to_checked            | 310  | Stuck   | 942   | pointer offset(8) on byte ptr in AggregateKindRawPtr                        |
| entrypoint::test_process_sync_native                | 590  | Stuck   | 1508  | 2 calls to raw_eq (with alloc as arg.), reads alloc (err case)              |
| entrypoint::test_process_get_account_data_size      | 105  | Stuck   | 319   | call to raw_eq (with alloc as arg.)                                         |
| entrypoint::test_process_initialize_immutable_owner | 410  | Stuck   | 1442  | call to core::result::unwrap_failed, reads alloc (2 err cases)              |
| entrypoint::test_process_amount_to_ui_amount        | 398  | Stuck   | 779   | call to raw_eq (with alloc as arg.)                                         |
| entrypoint::test_process_ui_amount_to_amount        | 251  | Stuck   | 90    | call to core::str::convert::from_utf8 (stdlib)                              |

The following tests are missing a cheat code setup:
(keep the first column so the `run-proofs.sh` script does not try to run these)

| Cheatcode |                                                    | Sec | Status  | Steps |                                                                 |
| Missing   | Start symbol name                                  |     |         |       |                                                                 |
|-----------|----------------------------------------------------|-----|---------|-------|-----------------------------------------------------------------|
|           | entrypoint::test_process_initialize_account        | 580 | Stuck   | <150  | 3 non-det. branches with #switchMatch, missing cheatcode        |
|           | entrypoint::test_process_initialize_account2       | 590 | Stuck   | <150  | 3 non-det. branches with #switchMatch, missing cheatcode        |
|           | entrypoint::test_process_initialize_mint_freeze    | 143 | Stuck   | 267   | missing cheatcode: thunk ptr offset (data_ptr access)           |
|           | entrypoint::test_process_initialize_mint_no_freeze | 142 | Stuck   | 267   | missing cheatcode: thunk ptr offset (data_ptr access)           |
|           | entrypoint::test_process_initialize_multisig       | 251 | Stuck   | 649   | Deref of allocated constant (with provenance)                   |
|           | entrypoint::test_process_approve                   | 133 | PASSED  | 373   | returns ProgramError::Custom(12)                                |
|           | entrypoint::test_process_set_authority             | 120 | PASSED  | 210   | returns ProgramError::Custom(12)                                |
|           | entrypoint::test_process_approve_checked           | 160 | PASSED  | 373   | returns ProgramError::Custom(12)                                |
|           | entrypoint::test_process_initialize_multisig2      | 600 | TIMEOUT | ~700  | 3 non-det. branches, call to assert_inhabited intrinsic         |
|           | entrypoint::test_process_withdraw_excess_lamports  | 600 | TIMEOUT | <200  | many branches, wrong local index(!), ptrMetadata on PAccByteRef |
