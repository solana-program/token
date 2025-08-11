| Start symbol name                            | Sec | Status  | Steps |                                            |
|----------------------------------------------|-----|---------|-------|--------------------------------------------|
| entrypoint::test_process_initialize_mint     | 79  | Stuck   | 243   | Return ref to ptr with offset thunk        |
| entrypoint::test_process_initialize_mint2    | 79  | Stuck   | 243   | Return ref to ptr with offset thunk        |
| entrypoint::test_process_initialize_account  | 122 | Stuck   | 283   | AggregateKindRawPtr with offset/cast thunk |
| entrypoint::test_process_transfer            | 198 | Stuck   | 713   | AggregateKindRawPtr with offset/cast thunk |
| entrypoint::test_process_mint_to             | 151 | Stuck   | 601   | AggregateKindRawPtr with offset/cast thunk |
| entrypoint::test_process_burn                | 147 | Stuck   | 601   | AggregateKindRawPtr with offset/cast thunk |
| entrypoint::test_process_close_account       | 300 | timeout | <=325 | non-det branches with #switchMatch         |
| entrypoint::test_process_transfer_checked    | 145 | Stuck   | 337   | AggregateKindRawPtr with offset            |
| entrypoint::test_process_burn_checked        | 125 | Stuck   | 337   | AggregateKindRawPtr with offset            |
| entrypoint::test_process_initialize_account2 | 118 | Stuck   | 286   | AggregateKindRawPtr with offset/cast thunk |
| entrypoint::test_process_initialize_account3 | 110 | Stuck   | 286   | AggregateKindRawPtr with offset/cast thunk |
