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
    echo(chalk.yellow('[ WARNING ]'), `Fixtures directory already exists.`);
  } else {
    await $`mkdir ${FIXTURES_DIR}`;

    // Fixtures are generated from the SPL Token program.
    cd(SPL_TOKEN_DIR);

    await $`RUST_LOG=error EJECT_FUZZ_FIXTURES=${FIXTURES_DIR} cargo test-sbf --features mollusk-svm/fuzz`;
  }
}

async function run(args) {
  if (!existsSync(FIXTURES_DIR)) {
    throw new Error(`Fixtures directory does not exist: ${FIXTURES_DIR}`);
  }

  const [programName] = args;
  if (!programName) {
    throw new Error('The name of the program file must be provided.');
  }

  await $`mollusk execute-fixture                     \
    ${path.join(SBF_OUTPUT_DIR, programName + '.so')} \
    ${FIXTURES_DIR}                                   \
    TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA       \
    --ignore-compute-units`;
}
