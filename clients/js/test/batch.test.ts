import { systemProgram } from '@solana-program/system';
import { generateKeyPairSigner, none, some } from '@solana/kit';
import { createLocalClient } from '@solana/kit-client-rpc';
import test from 'ava';
import { getMintSize, TOKEN_PROGRAM_ADDRESS, tokenProgram } from '../src';

test('it batches multiple token instructions together', async t => {
    // Given
    const client = await createLocalClient().use(systemProgram()).use(tokenProgram());
    const [mint, token, mintAuthority, tokenOwner] = await Promise.all([
        generateKeyPairSigner(),
        generateKeyPairSigner(),
        generateKeyPairSigner(),
        generateKeyPairSigner(),
    ]);
    const mintSize = getMintSize();
    const tokenSize = getMintSize();

    // When
    await client.sendTransaction([
        client.system.instructions.createAccount({
            newAccount: mint,
            space: mintSize,
            lamports: await client.getMinimumBalance(mintSize),
            programAddress: TOKEN_PROGRAM_ADDRESS,
        }),
        client.system.instructions.createAccount({
            newAccount: token,
            space: tokenSize,
            lamports: await client.getMinimumBalance(tokenSize),
            programAddress: TOKEN_PROGRAM_ADDRESS,
        }),
        client.token.instructions.batch([
            client.token.instructions.initializeMint2({
                mint: mint.address,
                decimals: 2,
                mintAuthority: mintAuthority.address,
            }),
            client.token.instructions.initializeAccount3({
                account: token.address,
                mint: mint.address,
                owner: tokenOwner.address,
            }),
            client.token.instructions.mintTo({
                mint: mint.address,
                token: token.address,
                mintAuthority: mintAuthority,
                amount: 123_45,
            }),
        ]),
    ]);

    // Then
    const mintAccount = await client.token.accounts.mint.fetch(mint.address);
    t.like(mintAccount.data, {
        mintAuthority: some(mintAuthority.address),
        supply: 123_45n,
        decimals: 2,
        isInitialized: true,
        freezeAuthority: none(),
    });

    // And
    const tokenAccount = await client.token.accounts.token.fetch(token.address);
    t.like(tokenAccount.data, {
        mint: mint.address,
        owner: tokenOwner.address,
        amount: 123_45n,
        isInitialized: true,
    });
});
