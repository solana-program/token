| Start symbol name                            | Sec | Status | Steps |                                            |
|----------------------------------------------|-----|--------|-------|--------------------------------------------|
| entrypoint::test_process_initialize_mint     | 80  | Stuck  | 252   | BUG: ref offset -1                         |
| entrypoint::test_process_initialize_mint2    | 80  | Stuck  | 252   | BUG: ref offset -1                         |
| entrypoint::test_process_initialize_account  | 122 | Stuck  | 283   | AggregateKindRawPtr with offset/cast thunk |
| entrypoint::test_process_transfer            | 198 | Stuck  | 713   | AggregateKindRawPtr with offset/cast thunk |
| entrypoint::test_process_mint_to             | 120 | Stuck  | 389   | subslice traversal from end, too short     |
| entrypoint::test_process_burn                | 122 | Stuck  | 389   | subslice traversal from end, too short     |
| entrypoint::test_process_close_account       | 73  | Stuck  | 98    | subslice traversal from end, too short     |
| entrypoint::test_process_transfer_checked    | 145 | Stuck  | 337   | AggregateKindRawPtr with offset/cast thunk |
| entrypoint::test_process_burn_checked        | 125 | Stuck  | 337   | AggregateKindRawPtr with offset/cast thunk |
| entrypoint::test_process_initialize_account2 | 118 | Stuck  | 286   | AggregateKindRawPtr with offset/cast thunk |
| entrypoint::test_process_initialize_account3 | 110 | Stuck  | 286   | AggregateKindRawPtr with offset/cast thunk |
