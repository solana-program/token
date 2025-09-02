#!/usr/bin/env zx
import 'zx/globals';
import { cliArguments, workingDirectory } from './utils.mjs';

// Extract program IDs
const programCargoPath = path.join(workingDirectory, 'program', 'Cargo.toml');
const pInterfacePath = path.join(workingDirectory, 'p-interface', 'src', 'lib.rs');
const programCargoContent = await fs.readFile(programCargoPath, 'utf-8');
const pInterfaceContent = await fs.readFile(pInterfacePath, 'utf-8');
const splProgramIdMatch = programCargoContent.match(/program-id = "([^"]+)"/);
const pinocchioProgramIdMatch = pInterfaceContent.match(/declare_id!\("([^"]+)"\)/);

if (!splProgramIdMatch || !pinocchioProgramIdMatch) {
    console.error('ERROR: Could not find program IDs');
    process.exit(1);
}

const splProgramId = splProgramIdMatch[1];
const pinocchioProgramId = pinocchioProgramIdMatch[1];

if (splProgramId !== pinocchioProgramId) {
    throw new Error("Ids were different")
};
console.log(`Token Program ID: ${splProgramId}`);

// Build the p-token program (not SPL from programs)
console.log('Building p-token program...');
await $`pnpm p-token:build`;
console.log('p-token program built successfully');

// Copy the p-token binary to location tests expect it with expected name
const pTokenBinary = path.join(workingDirectory, 'target', 'deploy', 'pinocchio_token_program.so');
const expectedBinary = path.join(workingDirectory, 'target', 'deploy', 'spl_token.so');
await $`cp "${pTokenBinary}" "${expectedBinary}"`;
console.log(`Copied ${pTokenBinary} to ${expectedBinary}`);

// Run the program tests against the p-token binary
console.log('Running program tests against p-token binary...');
const [_folder, ...args] = cliArguments();
const sbfOutDir = path.join(workingDirectory, 'target', 'deploy');
const manifestPath = path.join(workingDirectory, 'program', 'Cargo.toml');
const cargoArgs = args.includes('--') ? args.slice(args.indexOf('--') + 1) : [];
await $`RUST_LOG=error SBF_OUT_DIR=${sbfOutDir} cargo test --manifest-path ${manifestPath} ${cargoArgs}`;
console.log('All tests passed against p-token implementation');
