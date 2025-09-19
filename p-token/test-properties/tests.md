Proofs timings and steps with default settings from run-proofs.sh:
max-depth 200, max-iterations 30, timeout 20min (1200)

|                                         |      |         | Steps |                                                                                   |
| Start symbol name                       | Sec  | Status  | max   |                                                                                   |
|-----------------------------------------|------|---------|-------|-----------------------------------------------------------------------------------|
| test_process_initialize_mint_freeze     | 890  | Stuck   | 1011  | branch on float computation thunk, many overflow branches, pending                |
| test_process_initialize_mint_no_freeze  | 875  | Stuck   | 1011  | branch on float computation thunk, many overflow branches, pending                |
| test_process_initialize_account         | 970  | Stuck   | 1076  | call to core::result::unwrap_failed, float thunk, many overflow branches, pending |
| test_process_initialize_account2        | 1010 | Stuck   | 1174  | call to core::result::unwrap_failed, float thunk, many overflow branches, pending |
| test_process_transfer                   | 1200 | Timeout | 2955  | call to core::result::unwrap_failed, reads alloc (2 cases)                        |
| test_process_mint_to                    | 1050 | Stuck   | 2371  | call to core::result::unwrap_failed, reads alloc (2 cases)                        |
| test_process_burn                       | 692  | Stuck   | 1684  | call to raw_eq (with alloc as arg.), reads alloc (1 case)                         |
| test_process_close_account              | 520  | Stuck   | 1468  | call to raw_eq (with alloc as arg.), reads alloc                                  |
| test_process_transfer_checked           | 1080 | Stuck   | 1951  | ptr offset, reads alloc (1 case)                                                  |
| test_process_burn_checked               | 670  | Stuck   | 1684  | call to raw_eq (with alloc as arg.), reads alloc (1 case)                         |
| test_process_initialize_account3        | 700  | Stuck   | 610   | branching/stuck on thunked ptr cast, "ExposeAddress", assert_inhab, vacuous       |
| test_process_initialize_mint2_freeze    | 412  | Stuck   | 275   | branching/stuck on thunked ptr cast, "ExposeAddress", assert_inhab, vacuous       |
| test_process_initialize_mint2_no_freeze | 390  | Stuck   | 275   | branching/stuck on thunked ptr cast, "ExposeAddress", assert_inhab, vacuous       |
| test_process_revoke                     | 660  | Stuck   | 1824  | call to core::result::unwrap_failed, reads alloc (2 cases)                        |
| test_process_freeze_account             | 1120 | Stuck   | 1993  | various branches on key prefix values, reads alloc (1 case)                       |
| test_process_thaw_account               | 1100 | Stuck   | 1993  | various branches on key prefix values, reads alloc (1 case)                       |
| test_process_mint_to_checked            | 310  | Stuck   | 942   | pointer offset(8) on byte ptr in AggregateKindRawPtr                              |
| test_process_sync_native                | 590  | Stuck   | 1508  | 2 calls to raw_eq (with alloc as arg.), reads alloc (1 case)                      |
| test_process_get_account_data_size      | 105  | Stuck   | 319   | call to raw_eq (with alloc as arg.)                                               |
| test_process_initialize_immutable_owner | 410  | Stuck   | 1442  | call to core::result::unwrap_failed, reads alloc (2 cases)                        |
| test_process_amount_to_ui_amount        | 398  | Stuck   | 779   | call to raw_eq (with alloc as arg.)                                               |
| test_process_ui_amount_to_amount        | 251  | Stuck   | 90    | call to core::str::convert::from_utf8 (stdlib)                                    |

The following tests have not been prepared as property tests yet:
(keep the first column so the `run-proofs.sh` script does not try to run these)

| Missing | Start symbol name                     | Sec | Status  | Steps |                                                                 |
|---------|---------------------------------------|-----|---------|-------|-----------------------------------------------------------------|
|         | test_process_initialize_multisig      | 251 | Stuck   | 649   | Deref of allocated constant (with provenance)                   |
|         | test_process_approve                  | 133 | PASSED  | 373   | returns ProgramError::Custom(12)                                |
|         | test_process_set_authority            | 120 | PASSED  | 210   | returns ProgramError::Custom(12)                                |
|         | test_process_approve_checked          | 160 | PASSED  | 373   | returns ProgramError::Custom(12)                                |
|         | test_process_initialize_multisig2     | 600 | TIMEOUT | ~700  | 3 non-det. branches, call to assert_inhabited intrinsic         |
|         | test_process_withdraw_excess_lamports | 600 | TIMEOUT | <200  | many branches, wrong local index(!), ptrMetadata on PAccByteRef |
