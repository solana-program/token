#!/usr/bin/env zx
import 'zx/globals';
import { existsSync } from 'fs';
import { cliArguments, workingDirectory } from '../utils.mjs';

// Directory where the fixtures are generated.
const FIXTURES_DIR = path.join(workingDirectory, 'target', 'fixtures');
// Directory of the SPL Token program.
const SPL_TOKEN_DIR = path.join(workingDirectory, 'program');
// Directory of the SBF program.
const SBF_OUTPUT_DIR = path.join(workingDirectory, 'target', 'deploy');

const [command, ...args] = cliArguments();

switch (command) {
  case 'clean':
    await clean();
    break;
  case 'generate':
    await generate();
    break;
  case 'run':
    await run(args);
    break;
  default:
    throw new Error(`Unknown command: ${command}`);
}

async function clean() {
  await $`rm -rf ${FIXTURES_DIR}`;
}

async function generate() {
  if (existsSync(FIXTURES_DIR)) {
    throw new Error(`Fixtures directory already exist: ${FIXTURES_DIR}`);
  }

  await $`mkdir ${FIXTURES_DIR}`;

  // Fixtures are generated from the SPL Token program.
  cd(SPL_TOKEN_DIR);

  await $`RUST_LOG=error EJECT_FUZZ_FIXTURES=${FIXTURES_DIR} cargo test-sbf --features mollusk-svm/fuzz`;

  await fs.writeFile(
    path.join(FIXTURES_DIR, 'config.json'),
    '{"checks": [{ "programResult": null }, { "returnData": null }, {"allResultingAccounts": { "data": true, "executable": true, "lamports": true, "owner": true, "space": true }}]}'
  );
}

async function run(args) {
  if (!existsSync(FIXTURES_DIR)) {
    throw new Error(`Fixtures directory does not exist: ${FIXTURES_DIR}`);
  }

  const [programName] = args;
  if (!programName) {
    throw new Error('The name of the program file must be provided.');
  }

  if (
    (await $`mollusk execute-fixture ${path.join(SBF_OUTPUT_DIR, programName + '.so')} ${FIXTURES_DIR} TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA -c ${FIXTURES_DIR}/config.json`.pipe(
      $`grep 'FAIL'`
    ).exitCode) === 0
  ) {
    // If `grep` finds a match (exit code 0), it means we found failing fixtures.
    throw new Error('There are failing fixtures.');
  } else {
    echo(chalk.green('[ SUCCESS ]'), `All fixtures passed.`);
  }
}
