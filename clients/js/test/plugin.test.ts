import {
    Address,
    address,
    createClient,
    generateKeyPairSigner,
    lamports,
    SequentialInstructionPlan,
} from '@solana/kit';
import { litesvm } from '@solana/kit-plugin-litesvm';
import { airdropSigner, generatedSigner } from '@solana/kit-plugin-signer';
import { expect, it } from 'vitest';

import { tokenProgram } from '../src';

const CUSTOM_PROGRAM_ADDRESS = address('TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb');

const createCustomProgramClient = () =>
    createClient()
        .use(generatedSigner())
        .use(litesvm())
        .use(airdropSigner(lamports(1_000_000_000n)))
        .use(tokenProgram({ programAddress: { address: CUSTOM_PROGRAM_ADDRESS } }));

const getProgramAddresses = (plan: SequentialInstructionPlan): Address[] =>
    plan.plans.flatMap(child => (child.kind === 'single' ? [child.instruction.programAddress] : []));

it('targets the plugin program address in generated and custom instructions', async () => {
    // Given a client whose token plugin targets a custom program address.
    const client = await createCustomProgramClient();
    const [source, destination, mint, owner] = await Promise.all([
        generateKeyPairSigner(),
        generateKeyPairSigner(),
        generateKeyPairSigner(),
        generateKeyPairSigner(),
    ]);

    // When we build generated, batch and plan-based instructions.
    const transfer = client.token.instructions.transfer({
        source: source.address,
        destination: destination.address,
        authority: owner,
        amount: 1n,
    });
    const batch = client.token.instructions.batch([transfer]);
    const mintToATA = (await client.token.instructions.mintToATA({
        mint: mint.address,
        owner: owner.address,
        mintAuthority: owner,
        amount: 1n,
        decimals: 0,
    })) as SequentialInstructionPlan;

    // Then they all target the custom program address.
    expect(transfer.programAddress).toBe(CUSTOM_PROGRAM_ADDRESS);
    expect(batch.programAddress).toBe(CUSTOM_PROGRAM_ADDRESS);
    expect(getProgramAddresses(mintToATA)[1]).toBe(CUSTOM_PROGRAM_ADDRESS);
});

it('lets per-instruction program addresses take precedence over the plugin one', async () => {
    // Given a client whose token plugin targets a custom program address.
    const client = await createCustomProgramClient();
    const [source, destination, owner] = await Promise.all([
        generateKeyPairSigner(),
        generateKeyPairSigner(),
        generateKeyPairSigner(),
    ]);
    const override = (await generateKeyPairSigner()).address;

    // When we override the program address for a single instruction.
    const transfer = client.token.instructions.transfer(
        { source: source.address, destination: destination.address, authority: owner, amount: 1n },
        { programAddress: override },
    );

    // Then that instruction targets the override.
    expect(transfer.programAddress).toBe(override);
});
