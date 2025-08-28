| Start symbol name                                   | Sec | Status  | Steps |                                                                              |
|-----------------------------------------------------|-----|---------|-------|------------------------------------------------------------------------------|
| entrypoint::test_process_transfer                   | 600 | Timeout | >850  | stuck on using an alloc                                                      |
| entrypoint::test_process_mint_to                    |     | Stuck   | 1136  | inconsistent projection involving IAcc                                       |
| entrypoint::test_process_burn                       |     | Stuck   | 236   | AggregateKindClosure                                                         |
| entrypoint::test_process_close_account              |     | Stuck   | 655   | erratic branching (cheatcode rules) after #fromPAcc(#toPAcc(_)), reads alloc |
| entrypoint::test_process_transfer_checked           |     | Timeout | ~930  | stuck on access to an alloc (1 branch)                                       |
| entrypoint::test_process_burn_checked               |     | Stuck   | 236   | AggregateKindClosure                                                         |
| entrypoint::test_process_initialize_account3        |     |         | ~600  | branching/stuck on thunked ptr cast, "ExposeAddress", assert_inhab, vacuous  |
| entrypoint::test_process_initialize_mint2_freeze    |     | Stuck   | 275   | branching/stuck on thunked ptr cast, "ExposeAddress", assert_inhab, vacuous  |
| entrypoint::test_process_initialize_mint2_no_freeze |     | Stuck   | 275   | branching/stuck on thunked ptr cast, "ExposeAddress", assert_inhab, vacuous  |
| entrypoint::test_process_revoke                     | 600 | Timeout | 955   | non-det branch on cheatcodes (invalid read from stack), reads alloc          |
| entrypoint::test_process_freeze_account             | 600 | Timeout | 1455  | reads alloc (one branch)                                                     |
| entrypoint::test_process_thaw_account               | 600 | Timeout | 1455  | reads alloc (one branch)                                                     |
| entrypoint::test_process_mint_to_checked            |     | Stuck   | 942   | pointer offset(8) on byte ptr in AggregateKindRawPtr                         |
| entrypoint::test_process_sync_native                | 600 | Timeout | 167   | erratic branching (cheatcode rules) after #fromPAcc(#toPAcc(_))              |
| entrypoint::test_process_get_account_data_size      | 600 | Timeout | 224   | erratic branching (cheatcode rules) after #fromPAcc(#toPAcc(_))              |
| entrypoint::test_process_initialize_immutable_owner | 600 | Timeout | 1306  | erratic branching (cheatcode rules) after #fromPAcc(#toPAcc(_)), reads alloc |
| entrypoint::test_process_amount_to_ui_amount        | 600 | Timeout | 684   | erratic branching (cheatcode rules) after #fromPAcc(#toPAcc(_))              |
| entrypoint::test_process_ui_amount_to_amount        |     | Stuck   | 90    | call to core::str::convert::from_utf8 (stdlib)                               |

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
