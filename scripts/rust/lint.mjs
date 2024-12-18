#!/usr/bin/env zx
import 'zx/globals';
import {
  cliArguments,
  getToolchainArgument,
  popArgument,
  workingDirectory,
} from '../utils.mjs';

const [folder, ...args] = cliArguments();

// Configure additional arguments here, e.g.:
// ['--arg1', '--arg2', ...cliArguments()]
const lintArgs = [
  '-Zunstable-options',
  '--all-targets',
  '--all-features',
  '--',
  '--deny=warnings',
  '--deny=clippy::arithmetic_side_effects',
  ...args,
];

const fix = popArgument(lintArgs, '--fix');
const toolchain = getToolchainArgument('lint');
const manifestPath = path.join(workingDirectory, folder, 'Cargo.toml');

// Check the client using Clippy.
if (fix) {
  await $`cargo ${toolchain} clippy --manifest-path ${manifestPath} --fix ${lintArgs}`;
} else {
  await $`cargo ${toolchain} clippy --manifest-path ${manifestPath} ${lintArgs}`;
}
