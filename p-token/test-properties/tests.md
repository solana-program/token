| Start symbol name                                   | Sec | Status  | Steps   |                                                          |
|-----------------------------------------------------|-----|---------|---------|----------------------------------------------------------|
| entrypoint::test_process_initialize_account         | 122 | Stuck   | 283     | AggregateKindRawPtr with offset/cast thunk               |
| entrypoint::test_process_transfer                   | 150 | Stuck   | 208     | BUG wrong projection (Deref of Aggregate)                |
| entrypoint::test_process_mint_to                    | 112 | Stuck   | 114     | BUG wrong projection (Deref of Aggregate)                |
| entrypoint::test_process_burn                       | 165 | Stuck   | 605     | AggregateKindRawPtr with offset/cast thunk               |
| entrypoint::test_process_close_account              | 300 | Unknown | 231     | non-det branches with #selectBlock                       |
| entrypoint::test_process_transfer_checked           | 175 | Stuck   | 208     | BUG wrong projection (Deref of Aggregate)                |
| entrypoint::test_process_burn_checked               | 129 | Stuck   | 333     | AggregateKindRawPtr with offset thunk                    |
| entrypoint::test_process_initialize_account2        | 118 | Stuck   | 286     | AggregateKindRawPtr with offset/cast thunk               |
| entrypoint::test_process_initialize_account3        | 110 | Stuck   | 286     | AggregateKindRawPtr with offset/cast thunk               |
| entrypoint::test_process_initialize_mint_freeze     | 103 | Stuck   | 278     | AggregateKindRawPtr with offset/cast thunk               |
| entrypoint::test_process_initialize_mint_no_freeze  | 101 | Stuck   | 278     | AggregateKindRawPtr with offset/cast thunk               |
| entrypoint::test_process_initialize_mint2_freeze    | 87  | Stuck   | 118     | call to function 200099 (black_box intr)                 |
| entrypoint::test_process_initialize_mint2_no_freeze | 86  | Stuck   | 118     | call to function 200099 (black_box intr)                 |
| entrypoint::test_process_initialize_multisig        | 97  | Stuck   | 649     | Bug wrong projection (Deref of Constant)                 |
| entrypoint::test_process_approve                    | 93  | PASSED  | 373     | returns ProgramError::Custom(12)                         |
| entrypoint::test_process_revoke                     | 105 | Stuck   | 292     | AggregateKindRawPtr with offset/cast thunk               |
| entrypoint::test_process_set_authority              | 89  | PASSED  | 210     | returns ProgramError::Custom(12)                         |
| entrypoint::test_process_freeze_account             | 121 | Stuck   | 329     | AggregateKindRawPtr with offset/cast thunk               |
| entrypoint::test_process_thaw_account               | 122 | Stuck   | 325     | AggregateKindRawPtr with offset/cast thunk               |
| entrypoint::test_process_approve_checked            | 98  | PASSED  | 373     | returns ProgramError::Custom(12)                         |
| entrypoint::test_process_mint_to_checked            | 123 | Stuck   | 114     | BUG wrong projection (Deref of Aggregate)                |
| entrypoint::test_process_sync_native                | 286 | Stuck   | 416     | call to function 500952 (raw_eq intr)                     |
| entrypoint::test_process_initialize_multisig2       | 97  | Stuck   | 645     | call to function 200099 (black_box intr)                 |
| entrypoint::test_process_get_account_data_size      | 70  | Stuck   | 224     | call to function 500952 (raw_eq intr)                     |
| entrypoint::test_process_initialize_immutable_owner | 246 | Stuck   | 497     | AggregateKindRawPtr with offset/cast thunk               |
| entrypoint::test_process_amount_to_ui_amount        | 183 | PASSED  | 373     | returns ProgramError::Custom(12)                         |
| entrypoint::test_process_ui_amount_to_amount        | 102 | Stuck   | 44      | call to function 500385 (core::str::converts::from_utf8) |
| entrypoint::test_process_withdraw_excess_lamports   | 128 | Stuck   | 327     | AggregateKindRawPtr with offset/cast thunk               |
