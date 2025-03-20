#!/usr/bin/env zx
import 'zx/globals';
import { cliArguments, getCargo, workingDirectory } from '../utils.mjs';

const [folder, ...args] = cliArguments();
const manifestPath = path.join(workingDirectory, folder, 'Cargo.toml');
await $`cargo semver-checks --manifest-path ${manifestPath} ${args}`;
