#!/usr/bin/env zx
import 'zx/globals';
import { cliArguments, popArgument } from '../utils.mjs';

const args = cliArguments();

const fix = popArgument(args, '--fix');

if (fix) {
  await $`tombi format ${args}`;
} else {
  await $`tombi format --check ${args}`;
}
