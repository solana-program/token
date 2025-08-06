| Start symbol name                            | Sec | Status | Steps |                                            |
|----------------------------------------------|-----|--------|-------|--------------------------------------------|
| entrypoint::test_process_initialize_mint     | 140 | Stuck  | 252   | BUG: ref offset -1                         |
| entrypoint::test_process_initialize_mint2    | 79  | Stuck  | 252   | BUG: ref offset -1                         |
| entrypoint::test_process_initialize_account  | 118 | Stuck  | 283   | AggregateKindRawPtr with offset/cast thunk |
| entrypoint::test_process_transfer            | 160 | Stuck  | 412   | subslice traversal from end                |
| entrypoint::test_process_mint_to             | 118 | Stuck  | 389   | subslice traversal from end                |
| entrypoint::test_process_burn                | 121 | Stuck  | 389   | subslice traversal from end                |
| entrypoint::test_process_close_account       | 73  | Stuck  | 98    | subslice traversal from end                |
| entrypoint::test_process_transfer_checked    | 130 | Stuck  | 245   | AggregateKindRawPtr with ptr and int       |
| entrypoint::test_process_burn_checked        | 112 | Stuck  | 245   | AggregateKindRawPtr with ptr and int       |
| entrypoint::test_process_initialize_account2 | 118 | Stuck  | 286   | AggregateKindRawPtr with offset/cast thunk |
| entrypoint::test_process_initialize_account3 | 110 | Stuck  | 286   | AggregateKindRawPtr with offset/cast thunk |
