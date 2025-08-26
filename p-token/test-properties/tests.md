| Start symbol name                                   | Sec | Status  | Steps |                                                                 |
|-----------------------------------------------------|-----|---------|-------|-----------------------------------------------------------------|
| entrypoint::test_process_initialize_account         | 580 | Stuck   | <150  | 3 non-det. branches with #switchMatch, missing cheatcode        |
| entrypoint::test_process_transfer                   | 520 | Stuck   | <600  | missing cheatcode (access to data_ptr of Account)               |
| entrypoint::test_process_mint_to                    | 432 | Stuck   | <400  | 3 non-det branches with #switchMatch, missing cheatcode         |
| entrypoint::test_process_burn                       | 131 | Stuck   | 20    | #mkAggregate ( aggregateKindClosure                             |
| entrypoint::test_process_close_account              | 129 | Stuck   | 119   | missing cheatcode (access to data_ptr of Account)               |
| entrypoint::test_process_transfer_checked           | 600 | TIMEOUT | 539   | 3 non-det branches with #switchMatch, missing cheatcode         |
| entrypoint::test_process_burn_checked               | 131 | Stuck   | 20    | #mkAggregate ( aggregateKindClosure                             |
| entrypoint::test_process_initialize_account2        | 590 | Stuck   | <150  | 3 non-det. branches with #switchMatch, missing cheatcode        |
| entrypoint::test_process_initialize_account3        | 340 | Stuck   | 287   | 3 non-det. branches with #switchMatch, missing cheatcode        |
| entrypoint::test_process_initialize_mint_freeze     | 143 | Stuck   | 267   | thunk ptr offset (data_ptr access)                              |
| entrypoint::test_process_initialize_mint_no_freeze  | 142 | Stuck   | 267   | thunk ptr offset (data_ptr access)                              |
| entrypoint::test_process_initialize_mint2_freeze    | 300 | Stuck   | 225   | 3 non-det branches, call to assert_inhabited intrinsic          |
| entrypoint::test_process_initialize_mint2_no_freeze | 280 | Stuck   | <300  | 3 non-det branches, call to assert_inhabited intrinsic          |
| entrypoint::test_process_initialize_multisig        | 251 | Stuck   | 649   | Deref of allocated constant (with provenance)                   |
| entrypoint::test_process_approve                    | 133 | PASSED  | 373   | returns ProgramError::Custom(12)                                |
| entrypoint::test_process_revoke                     | 123 | Stuck   | 119   | missing cheatcode (access to data_ptr of Account)               |
| entrypoint::test_process_set_authority              | 120 | PASSED  | 210   | returns ProgramError::Custom(12)                                |
| entrypoint::test_process_freeze_account             | 159 | Stuck   | 119   | missing cheatcode (access to data_ptr of Account)               |
| entrypoint::test_process_thaw_account               | 140 | Stuck   | 119   | missing cheatcode (access to data_ptr of Account)               |
| entrypoint::test_process_approve_checked            | 160 | PASSED  | 373   | returns ProgramError::Custom(12)                                |
| entrypoint::test_process_mint_to_checked            | 460 | Stuck   | <400  | 3 non-det. branches with #switchMatch, missing cheatcode        |
| entrypoint::test_process_sync_native                | 123 | Stuck   | 209   | missing cheatcode (access to data_ptr of Account)               |
| entrypoint::test_process_initialize_multisig2       | 600 | TIMEOUT | ~700  | 3 non-det. branches, call to assert_inhabited intrinsic         |
| entrypoint::test_process_get_account_data_size      | 600 | TIMEOUT |       | many non-det branches (PAcc access), call to raw_eq intrinsic   |
| entrypoint::test_process_initialize_immutable_owner | 96  | Stuck   | 119   | missing cheatcode (access to data_ptr of Account)               |
| entrypoint::test_process_amount_to_ui_amount        | 85  | Stuck   | 699   | call to raw_eq intrinsic                                        |
| entrypoint::test_process_ui_amount_to_amount        | 72  | Stuck   | 11    | call to function 500385 (core::str::converts::from_utf8)        |
| entrypoint::test_process_withdraw_excess_lamports   | 600 | TIMEOUT | <200  | many branches, wrong local index(!), ptrMetadata on PAccByteRef |
