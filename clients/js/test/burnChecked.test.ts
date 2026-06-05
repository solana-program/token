import { generateKeyPairSigner } from '@solana/kit';
import { expect, it } from 'vitest';
import { fetchMint, fetchToken } from '../src';
import { createTestClient, createTokenWithAmount } from './_setup';

it('burns tokens with correct decimals', async () => {
    // Given a mint with 9 decimals and a token account with 200 tokens.
    const client = await createTestClient();
    const [mintAuthority, owner, mint] = await Promise.all([
        generateKeyPairSigner(),
        generateKeyPairSigner(),
        generateKeyPairSigner(),
    ]);
    await client.token.instructions
        .createMint({ newMint: mint, decimals: 9, mintAuthority: mintAuthority.address })
        .sendTransaction();
    const token = await createTokenWithAmount(client, mintAuthority, mint.address, owner.address, 200n);

    // When we burn 25 tokens with the correct decimals (9).
    await client.token.instructions
        .burnChecked({ account: token, mint: mint.address, authority: owner, amount: 25n, decimals: 9 })
        .sendTransaction();

    const { data: mintData } = await fetchMint(client.rpc, mint.address);
    expect(mintData.supply).toBe(175n);

    // Then we expect the token account to have 175 tokens remaining.
    const { data: tokenData } = await fetchToken(client.rpc, token);
    expect(tokenData.amount).toBe(175n);
});

it('burns tokens using a delegate', async () => {
    // Given a token account with 100 tokens and a delegate approved for 60 tokens.
    const client = await createTestClient();
    const [mintAuthority, owner, delegate, mint] = await Promise.all([
        generateKeyPairSigner(),
        generateKeyPairSigner(),
        generateKeyPairSigner(),
        generateKeyPairSigner(),
    ]);
    await client.token.instructions
        .createMint({ newMint: mint, decimals: 9, mintAuthority: mintAuthority.address })
        .sendTransaction();
    const token = await createTokenWithAmount(client, mintAuthority, mint.address, owner.address, 100n);

    // Approve delegate for 60 tokens.
    await client.token.instructions
        .approve({ source: token, delegate: delegate.address, owner, amount: 60n })
        .sendTransaction();

    // When the delegate burns 30 tokens.
    await client.token.instructions
        .burnChecked({ account: token, mint: mint.address, authority: delegate, amount: 30n, decimals: 9 })
        .sendTransaction();

    // Then the token account should have 70 tokens remaining.
    const { data: tokenData } = await fetchToken(client.rpc, token);
    expect(tokenData.amount).toBe(70n);
    expect(tokenData.delegatedAmount).toBe(30n); // Remaining delegated amount
});

it('fails when decimals mismatch', async () => {
    // Given a mint with 9 decimals and a token account with tokens.
    const client = await createTestClient();
    const [mintAuthority, owner, mint] = await Promise.all([
        generateKeyPairSigner(),
        generateKeyPairSigner(),
        generateKeyPairSigner(),
    ]);
    await client.token.instructions
        .createMint({ newMint: mint, decimals: 9, mintAuthority: mintAuthority.address })
        .sendTransaction();
    const token = await createTokenWithAmount(client, mintAuthority, mint.address, owner.address, 100n);

    // When we try to burn with incorrect decimals (6 instead of 9).
    // Then it should fail with MintDecimalsMismatch error.
    await expect(
        client.token.instructions
            .burnChecked({ account: token, mint: mint.address, authority: owner, amount: 40n, decimals: 6 })
            .sendTransaction(),
    ).rejects.toThrow();
});

it('fails when burning more than account balance', async () => {
    // Given a token account with only 50 tokens.
    const client = await createTestClient();
    const [mintAuthority, owner, mint] = await Promise.all([
        generateKeyPairSigner(),
        generateKeyPairSigner(),
        generateKeyPairSigner(),
    ]);
    await client.token.instructions
        .createMint({ newMint: mint, decimals: 9, mintAuthority: mintAuthority.address })
        .sendTransaction();
    const token = await createTokenWithAmount(client, mintAuthority, mint.address, owner.address, 50n);

    // When we try to burn 150 tokens (more than available).
    // Then it should fail with InsufficientFunds error.
    await expect(
        client.token.instructions
            .burnChecked({ account: token, mint: mint.address, authority: owner, amount: 150n, decimals: 9 })
            .sendTransaction(),
    ).rejects.toThrow();
});

it('fails when authority is not a signer', async () => {
    // Given a token account with tokens.
    const client = await createTestClient();
    const [mintAuthority, owner, wrongAuthority, mint] = await Promise.all([
        generateKeyPairSigner(),
        generateKeyPairSigner(),
        generateKeyPairSigner(),
        generateKeyPairSigner(),
    ]);
    await client.token.instructions
        .createMint({ newMint: mint, decimals: 9, mintAuthority: mintAuthority.address })
        .sendTransaction();
    const token = await createTokenWithAmount(client, mintAuthority, mint.address, owner.address, 100n);

    // When we try to burn with wrong authority (not the owner).
    // Then it should fail (owner mismatch or missing signature).
    await expect(
        client.token.instructions
            .burnChecked({ account: token, mint: mint.address, authority: wrongAuthority, amount: 40n, decimals: 9 })
            .sendTransaction(),
    ).rejects.toThrow();
});

it('fails when delegate has insufficient delegated amount', async () => {
    // Given a token account with 100 tokens and a delegate approved for only 20 tokens.
    const client = await createTestClient();
    const [mintAuthority, owner, delegate, mint] = await Promise.all([
        generateKeyPairSigner(),
        generateKeyPairSigner(),
        generateKeyPairSigner(),
        generateKeyPairSigner(),
    ]);
    await client.token.instructions
        .createMint({ newMint: mint, decimals: 9, mintAuthority: mintAuthority.address })
        .sendTransaction();
    const token = await createTokenWithAmount(client, mintAuthority, mint.address, owner.address, 100n);

    // Approve delegate for only 20 tokens.
    await client.token.instructions
        .approve({ source: token, delegate: delegate.address, owner, amount: 20n })
        .sendTransaction();

    // When the delegate tries to burn 50 tokens (more than delegated).
    // Then it should fail with InsufficientFunds error.
    await expect(
        client.token.instructions
            .burnChecked({ account: token, mint: mint.address, authority: delegate, amount: 50n, decimals: 9 })
            .sendTransaction(),
    ).rejects.toThrow();
});

it('burns zero tokens successfully', async () => {
    // Given a token account with tokens.
    const client = await createTestClient();
    const [mintAuthority, owner, mint] = await Promise.all([
        generateKeyPairSigner(),
        generateKeyPairSigner(),
        generateKeyPairSigner(),
    ]);
    await client.token.instructions
        .createMint({ newMint: mint, decimals: 9, mintAuthority: mintAuthority.address })
        .sendTransaction();
    const token = await createTokenWithAmount(client, mintAuthority, mint.address, owner.address, 100n);

    // When we burn 0 tokens (edge case).
    await client.token.instructions
        .burnChecked({ account: token, mint: mint.address, authority: owner, amount: 0n, decimals: 9 })
        .sendTransaction();

    // Then the balance should remain unchanged.
    const { data: tokenData } = await fetchToken(client.rpc, token);
    expect(tokenData.amount).toBe(100n);
});
