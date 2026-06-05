# JavaScript client

A generated JavaScript library for the Token program.

## Getting started

The JS client tests use [LiteSVM](https://github.com/LiteSVM/litesvm) in-process, so no local validator is needed. To build and test your JavaScript client from the root of the repository, you may use the following command.

```sh
make test-js-clients-js
```

This installs dependencies, builds the client, and runs the test suite.

## Available client scripts.

Alternatively, you can go into the client directory and run the tests directly.

```sh
# Build the program `.so` that LiteSVM loads.
make build-sbf-pinocchio-program

# Go into the client directory and run the tests.
cd clients/js
pnpm install
pnpm build
pnpm test
```

You may also use the following scripts to lint and/or format your JavaScript client.

```sh
pnpm lint
pnpm lint:fix
pnpm format
pnpm format:fix
```
