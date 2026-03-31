import { Account, generateKeyPairSigner } from '@solana/kit';
import { createLocalClient } from '@solana/kit-client-rpc';
import test from 'ava';
import { AccountState, fetchToken, findAssociatedTokenPda, Token, TOKEN_PROGRAM_ADDRESS, tokenProgram } from '../src';
import {
    createDefaultSolanaClient,
    createMint,
    createTokenPdaWithAmount,
    generateKeyPairSignerWithSol,
} from './_setup';

test('plugin mintToATA defaults payer and auto-derives ATA', async t => {
    // Given a mint account, its mint authority and a token owner.
    const client = await createLocalClient().use(tokenProgram());
    const mintAuthority = await generateKeyPairSigner();
    const owner = await generateKeyPairSigner();
    const mint = await generateKeyPairSigner();

    // And a mint created via the plugin.
    await client.token.instructions
        .createMint({ newMint: mint, decimals: 2, mintAuthority: mintAuthority.address })
        .sendTransaction();

    // When we mint to the owner via the plugin (payer defaulted, ATA derived).
    await client.token.instructions
        .mintToATA({
            mint: mint.address,
            owner: owner.address,
            mintAuthority,
            amount: 1_000n,
            decimals: 2,
        })
        .sendTransaction();

    // Then we expect the derived ATA to exist with the correct balance.
    const [ata] = await findAssociatedTokenPda({
        mint: mint.address,
        owner: owner.address,
        tokenProgram: TOKEN_PROGRAM_ADDRESS,
    });

    t.like(await fetchToken(client.rpc, ata), <Account<Token>>{
        address: ata,
        data: {
            mint: mint.address,
            owner: owner.address,
            amount: 1000n,
            state: AccountState.Initialized,
        },
    });
});

test('plugin transferToATA defaults payer and auto-derives source + destination', async t => {
    // Given a mint account and ownerA's ATA with 100 tokens.
    const baseClient = createDefaultSolanaClient();
    const [payer, mintAuthority, ownerA, ownerB] = await Promise.all([
        generateKeyPairSignerWithSol(baseClient),
        generateKeyPairSigner(),
        generateKeyPairSigner(),
        generateKeyPairSigner(),
    ]);
    const decimals = 2;
    const mint = await createMint(baseClient, payer, mintAuthority.address, decimals);
    await createTokenPdaWithAmount(baseClient, payer, mintAuthority, mint, ownerA.address, 100n, decimals);

    // When ownerA transfers 50 tokens to ownerB via the plugin (payer defaulted, source + destination derived).
    const client = await createLocalClient().use(tokenProgram());
    await client.token.instructions
        .transferToATA({
            mint,
            authority: ownerA,
            recipient: ownerB.address,
            amount: 50n,
            decimals,
        })
        .sendTransaction();

    // Then we expect both ATAs to have the correct balances.
    const [sourceAta] = await findAssociatedTokenPda({
        owner: ownerA.address,
        mint,
        tokenProgram: TOKEN_PROGRAM_ADDRESS,
    });
    const [destAta] = await findAssociatedTokenPda({
        owner: ownerB.address,
        mint,
        tokenProgram: TOKEN_PROGRAM_ADDRESS,
    });

    t.like(await fetchToken(client.rpc, sourceAta), <Account<Token>>{
        data: { amount: 50n },
    });
    t.like(await fetchToken(client.rpc, destAta), <Account<Token>>{
        data: { amount: 50n },
    });
});
