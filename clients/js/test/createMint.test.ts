import { Account, generateKeyPairSigner, none, some } from '@solana/kit';
import { createDefaultLocalhostRpcClient } from '@solana/kit-plugins';
import test from 'ava';
import { fetchMint, getCreateMintInstructionPlan, Mint, tokenProgram } from '../src';
import { createDefaultSolanaClient, createDefaultTransactionPlanner, generateKeyPairSignerWithSol } from './_setup';

test('it creates and initializes a new mint account', async t => {
    // Given an authority and a mint account.
    const client = createDefaultSolanaClient();
    const authority = await generateKeyPairSignerWithSol(client);
    const mint = await generateKeyPairSigner();

    // When we create and initialize a mint account at this address.
    const instructionPlan = getCreateMintInstructionPlan({
        payer: authority,
        newMint: mint,
        decimals: 2,
        mintAuthority: authority.address,
    });

    const transactionPlanner = createDefaultTransactionPlanner(client, authority);
    const transactionPlan = await transactionPlanner(instructionPlan);
    await client.sendTransactionPlan(transactionPlan);

    // Then we expect the mint account to exist and have the following data.
    const mintAccount = await fetchMint(client.rpc, mint.address);
    t.like(mintAccount, <Account<Mint>>{
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

test('it creates a new mint account with a freeze authority', async t => {
    // Given an authority and a mint account.
    const client = createDefaultSolanaClient();
    const [payer, mintAuthority, freezeAuthority, mint] = await Promise.all([
        generateKeyPairSignerWithSol(client),
        generateKeyPairSigner(),
        generateKeyPairSigner(),
        generateKeyPairSigner(),
    ]);

    // When we create and initialize a mint account at this address.
    const instructionPlan = getCreateMintInstructionPlan({
        payer: payer,
        newMint: mint,
        decimals: 2,
        mintAuthority: mintAuthority.address,
        freezeAuthority: freezeAuthority.address,
    });

    const transactionPlanner = createDefaultTransactionPlanner(client, payer);
    const transactionPlan = await transactionPlanner(instructionPlan);
    await client.sendTransactionPlan(transactionPlan);

    // Then we expect the mint account to exist and have the following data.
    const mintAccount = await fetchMint(client.rpc, mint.address);
    t.like(mintAccount, <Account<Mint>>{
        address: mint.address,
        data: {
            mintAuthority: some(mintAuthority.address),
            freezeAuthority: some(freezeAuthority.address),
        },
    });
});

test('it creates and initializes a new mint account using the token program plugin', async t => {
    // Given a client with the token program plugin, and a mint account.
    const client = await createDefaultLocalhostRpcClient().use(tokenProgram());
    const mint = await generateKeyPairSigner();

    // When we send the "create mint" instruction plan.
    await client.token.instructions
        .createMint({ newMint: mint, decimals: 2, mintAuthority: client.payer.address })
        .sendTransaction();

    // Then we expect the mint account to exist and have the following data.
    const mintAccount = await client.token.accounts.mint.fetch(mint.address);
    t.like(mintAccount, {
        address: mint.address,
        data: {
            mintAuthority: some(client.payer.address),
            supply: 0n,
            decimals: 2,
            isInitialized: true,
            freezeAuthority: none(),
        },
    });
});
