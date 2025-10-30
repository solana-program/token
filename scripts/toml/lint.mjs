#!/usr/bin/env zx
import 'zx/globals';
import { cliArguments } from '../utils.mjs';

const args = cliArguments();

await $`tombi lint ${args}`;
