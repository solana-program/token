import { Account, generateKeyPairSigner, none, some } from '@solana/kit';
import { expect, it } from 'vitest';
import { Mint, TOKEN_PROGRAM_ADDRESS, fetchMint, getMintSize } from '../src';
import { createTestClient } from './_setup';

it('creates and initializes a new mint account', async () => {
    // Given an authority and a mint account.
    const client = await createTestClient();
    const [authority, mint] = await Promise.all([generateKeyPairSigner(), generateKeyPairSigner()]);

    // When we create and initialize a mint account at this address.
    const space = BigInt(getMintSize());
    const rent = await client.rpc.getMinimumBalanceForRentExemption(space).send();
    await client.sendTransaction([
        client.system.instructions.createAccount({
            newAccount: mint,
            lamports: rent,
            space,
            programAddress: TOKEN_PROGRAM_ADDRESS,
        }),
        client.token.instructions.initializeMint({
            mint: mint.address,
            decimals: 2,
            mintAuthority: authority.address,
        }),
    ]);

    // Then we expect the mint account to exist and have the following data.
    const mintAccount = await fetchMint(client.rpc, mint.address);
    expect(mintAccount).toMatchObject(<Account<Mint>>{
        address: mint.address,
        data: {
            mintAuthority: some(authority.address),
            supply: 0n,
            decimals: 2,
            isInitialized: true,
            freezeAuthority: none(),
        },
    });
});

it('creates a new mint account with a freeze authority', async () => {
    // Given an authority and a mint account.
    const client = await createTestClient();
    const [mintAuthority, freezeAuthority, mint] = await Promise.all([
        generateKeyPairSigner(),
        generateKeyPairSigner(),
        generateKeyPairSigner(),
    ]);

    // When we create and initialize a mint account at this address.
    const space = BigInt(getMintSize());
    const rent = await client.rpc.getMinimumBalanceForRentExemption(space).send();
    await client.sendTransaction([
        client.system.instructions.createAccount({
            newAccount: mint,
            lamports: rent,
            space,
            programAddress: TOKEN_PROGRAM_ADDRESS,
        }),
        client.token.instructions.initializeMint({
            mint: mint.address,
            decimals: 0,
            mintAuthority: mintAuthority.address,
            freezeAuthority: freezeAuthority.address,
        }),
    ]);

    // Then we expect the mint account to exist and have the following data.
    const mintAccount = await fetchMint(client.rpc, mint.address);
    expect(mintAccount).toMatchObject(<Account<Mint>>{
        address: mint.address,
        data: {
            mintAuthority: some(mintAuthority.address),
            freezeAuthority: some(freezeAuthority.address),
        },
    });
});
