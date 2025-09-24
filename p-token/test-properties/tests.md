Proofs timings and steps with default settings from run-proofs.sh:
max-depth 300, max-iterations 100, timeout 45 min (2700)


| Start symbol name                       | Sec  | Status  | max steps |                                                                               |
|-----------------------------------------|------|---------|-----------|-------------------------------------------------------------------------------|
| test_process_approve                    | 590  | Stuck   | 715       | call to `unwrap_failed` (**unconditional**)                                   |
| test_process_approve_checked            | 640  | Stuck   | 794       | call to `unwrap_failed` (**unconditional**)                                   |
| test_process_withdraw_excess_lamports   | 1300 | Stuck   | 1201      | reads alloc(AcctState), **projection error on cheat code data**               |
| test_process_initialize_mint_freeze     | 2450 | Stuck   | 1064      | 3x Overflow (Rent), 4x stuck on (trivial) ptr offset                          |
| test_process_initialize_mint_no_freeze  | 2450 | Stuck   | 1064      | 3x Overflow (Rent), 4x stuck on (trivial) ptr offset                          |
| test_process_initialize_account         | 2700 | Timeout | 2182      | `unwrap_failed`, 3x Overflow (Rent), Float + key comparison branches, pending |
| test_process_initialize_account2        | 2700 | Timeout | 2282      | `unwrap_failed`, 3x Overflow (Rent), Float + key comparison branches, pending |
| test_process_transfer                   | 2700 | Timeout | 2294      | 2x reads alloc(AcctState), **non-det branch on DELEGATE key**, pending        |
| test_process_mint_to                    | 660  | Stuck   | 845       | call to `unwrap_failed` (**unconditional**)                                   |
| test_process_burn                       | 2700 | Timeout | 2510      | reads alloc(AcctState), **non-det branch on DELEGATE key**, pending           |
| test_process_close_account              | 2650 | Stuck   | 2409      | reads alloc(AcctState), 4x `unwrap_failed` **unconditional**                  |
| test_process_transfer_checked           | 2700 | Timeout | 2483      | 2x reads alloc (AcctState), **non-det branch on DELEGATE key**, pending       |
| test_process_burn_checked               | 2700 | Timeout | 2510      | 2x reads alloc (AcctState), **non-det branch on DELEGATE key**, pending       |
| test_process_initialize_account3        | 2070 | Stuck   | 628       | branching on thunked ptr cast, "ExposeAddress", `assert_inhab`, vacuous       |
| test_process_initialize_mint2_freeze    | 2120 | Stuck   | 275       | branching on thunked ptr cast, "ExposeAddress", `assert_inhab`, vacuous       |
| test_process_initialize_mint2_no_freeze | 2150 | Stuck   | 275       | branching on thunked ptr cast, "ExposeAddress", `assert_inhab`, vacuous       |
| test_process_revoke                     | 970  | Stuck   | 1124      | call to `unwrap_failed`, reads alloc                                          |
| test_process_freeze_account             | 2480 | Stuck   | 1888      | reads alloc(AcctState), 4x `unwrap_failed` **unconditional**                  |
| test_process_thaw_account               | 2460 | Stuck   | 1888      | reads alloc(AcctState), 4x `unwrap_failed` **unconditional**                  |
| test_process_mint_to_checked            | 680  | Stuck   | 845       | call to `unwrap_failed` (**unconditional**)                                   |
| test_process_sync_native                | 2480 | Stuck   | 2776      | reads allocs (AcctState, 2x PrgErr), 2x `unwrap_failed`                       |
| test_process_get_account_data_size      | 1270 | Stuck   | 1928      | reads allocs (2x PrgResult), _terminates on 1 branch_                         |
| test_process_initialize_immutable_owner | 840  | Stuck   | 1425      | reads alloc(AcctState), call to `unwrap_failed`                               |
| test_process_amount_to_ui_amount        | 2580 | Stuck   | 2430      | reads allocs(2x PrgResult), stuck on (trivial) ptr offset                     |
| test_process_ui_amount_to_amount        | 580  | Stuck   | 90        | call to `core::str::convert::from_utf8` (stdlib)                              |

Cheat codes are missing or a problem for these proofs, therefore not recommended to execute them
(keep the empty first column so `run-proofs.sh` won't pick these up):

|   | test_process_initialize_multisig  |
|   | test_process_initialize_multisig2 |
|   | test_process_set_authority        |
