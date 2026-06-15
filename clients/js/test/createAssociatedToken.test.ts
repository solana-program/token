import { Account, generateKeyPairSigner, none } from '@solana/kit';
import { expect, it } from 'vitest';

import { AccountState, TOKEN_PROGRAM_ADDRESS, Token, fetchToken, findAssociatedTokenPda } from '../src';
import { createTestClient } from './_setup';

it('creates a new associated token account', async () => {
    // Given a mint account, its mint authority and a token owner.
    const client = await createTestClient();
    const [mintAuthority, owner, mint] = await Promise.all([
        generateKeyPairSigner(),
        generateKeyPairSigner(),
        generateKeyPairSigner(),
    ]);
    await client.token.instructions
        .createMint({ newMint: mint, decimals: 0, mintAuthority: mintAuthority.address })
        .sendTransaction();

    // When we create and initialize a token account at this address.
    await client.associatedToken.instructions
        .createAssociatedToken({
            mint: mint.address,
            owner: owner.address,
        })
        .sendTransaction();

    // Then we expect the token account to exist and have the following data.
    const [ata] = await findAssociatedTokenPda({
        mint: mint.address,
        owner: owner.address,
        tokenProgram: TOKEN_PROGRAM_ADDRESS,
    });
    expect(await fetchToken(client.rpc, ata)).toMatchObject(<Account<Token>>{
        address: ata,
        data: {
            mint: mint.address,
            owner: owner.address,
            amount: 0n,
            delegate: none(),
            state: AccountState.Initialized,
            isNative: none(),
            delegatedAmount: 0n,
            closeAuthority: none(),
        },
    });
});
