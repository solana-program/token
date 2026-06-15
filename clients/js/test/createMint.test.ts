import { Account, generateKeyPairSigner, none, some } from '@solana/kit';
import { expect, it } from 'vitest';

import { fetchMint, Mint } from '../src';
import { createTestClient } from './_setup';

it('creates and initializes a new mint account', async () => {
    // Given an authority and a mint account.
    const client = await createTestClient();
    const [authority, mint] = await Promise.all([generateKeyPairSigner(), generateKeyPairSigner()]);

    // When we create and initialize a mint account at this address.
    await client.token.instructions
        .createMint({ newMint: mint, decimals: 2, mintAuthority: authority.address })
        .sendTransaction();

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
    await client.token.instructions
        .createMint({
            newMint: mint,
            decimals: 2,
            mintAuthority: mintAuthority.address,
            freezeAuthority: freezeAuthority.address,
        })
        .sendTransaction();

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
