import { appendTransactionMessageInstruction, generateKeyPairSigner, pipe } from '@solana/kit';
import test from 'ava';
import { fetchMint, fetchToken, getApproveInstruction, getBurnCheckedInstruction } from '../src';
import {
    createDefaultSolanaClient,
    createDefaultTransaction,
    createMint,
    createTokenWithAmount,
    generateKeyPairSignerWithSol,
    signAndSendTransaction,
} from './_setup';

test('it burns tokens with correct decimals', async t => {
    // Given a mint with 9 decimals and a token account with 200 tokens.
    const client = createDefaultSolanaClient();
    const [payer, mintAuthority, owner] = await Promise.all([
        generateKeyPairSignerWithSol(client),
        generateKeyPairSigner(),
        generateKeyPairSigner(),
    ]);
    const mint = await createMint(client, payer, mintAuthority.address, 9);
    const token = await createTokenWithAmount(client, payer, mintAuthority, mint, owner.address, 200n);

    // When we burn 25 tokens with the correct decimals (9).
    const burnChecked = getBurnCheckedInstruction({
        account: token,
        mint,
        authority: owner,
        amount: 25n,
        decimals: 9,
    });
    await pipe(
        await createDefaultTransaction(client, payer),
        tx => appendTransactionMessageInstruction(burnChecked, tx),
        tx => signAndSendTransaction(client, tx),
    );

    const { data: mintData } = await fetchMint(client.rpc, mint);
    t.is(mintData.supply, 175n);

    // Then we expect the token account to have 175 tokens remaining.
    const { data: tokenData } = await fetchToken(client.rpc, token);
    t.is(tokenData.amount, 175n);
});

test('it burns tokens using a delegate', async t => {
    // Given a token account with 100 tokens and a delegate approved for 60 tokens.
    const client = createDefaultSolanaClient();
    const [payer, mintAuthority, owner, delegate] = await Promise.all([
        generateKeyPairSignerWithSol(client),
        generateKeyPairSigner(),
        generateKeyPairSigner(),
        generateKeyPairSigner(),
    ]);
    const mint = await createMint(client, payer, mintAuthority.address, 9);
    const token = await createTokenWithAmount(client, payer, mintAuthority, mint, owner.address, 100n);

    // Approve delegate for 60 tokens.
    const approve = getApproveInstruction({
        source: token,
        delegate: delegate.address,
        owner,
        amount: 60n,
    });
    await pipe(
        await createDefaultTransaction(client, payer),
        tx => appendTransactionMessageInstruction(approve, tx),
        tx => signAndSendTransaction(client, tx),
    );

    // When the delegate burns 30 tokens.
    const burnChecked = getBurnCheckedInstruction({
        account: token,
        mint,
        authority: delegate,
        amount: 30n,
        decimals: 9,
    });
    await pipe(
        await createDefaultTransaction(client, payer),
        tx => appendTransactionMessageInstruction(burnChecked, tx),
        tx => signAndSendTransaction(client, tx),
    );

    // Then the token account should have 70 tokens remaining.
    const { data: tokenData } = await fetchToken(client.rpc, token);
    t.is(tokenData.amount, 70n);
    t.is(tokenData.delegatedAmount, 30n); // Remaining delegated amount
});

test('it fails when decimals mismatch', async t => {
    // Given a mint with 9 decimals and a token account with tokens.
    const client = createDefaultSolanaClient();
    const [payer, mintAuthority, owner] = await Promise.all([
        generateKeyPairSignerWithSol(client),
        generateKeyPairSigner(),
        generateKeyPairSigner(),
    ]);
    const mint = await createMint(client, payer, mintAuthority.address, 9);
    const token = await createTokenWithAmount(client, payer, mintAuthority, mint, owner.address, 100n);

    // When we try to burn with incorrect decimals (6 instead of 9).
    const burnChecked = getBurnCheckedInstruction({
        account: token,
        mint,
        authority: owner,
        amount: 40n,
        decimals: 6, // Wrong! Should be 9
    });
    const transactionMessage = await pipe(await createDefaultTransaction(client, payer), tx =>
        appendTransactionMessageInstruction(burnChecked, tx),
    );

    // Then it should fail with MintDecimalsMismatch error.
    await t.throwsAsync(signAndSendTransaction(client, transactionMessage));
});

test('it fails when burning more than account balance', async t => {
    // Given a token account with only 50 tokens.
    const client = createDefaultSolanaClient();
    const [payer, mintAuthority, owner] = await Promise.all([
        generateKeyPairSignerWithSol(client),
        generateKeyPairSigner(),
        generateKeyPairSigner(),
    ]);
    const mint = await createMint(client, payer, mintAuthority.address, 9);
    const token = await createTokenWithAmount(client, payer, mintAuthority, mint, owner.address, 50n);

    // When we try to burn 150 tokens (more than available).
    const burnChecked = getBurnCheckedInstruction({
        account: token,
        mint,
        authority: owner,
        amount: 150n,
        decimals: 9,
    });
    const transactionMessage = await pipe(await createDefaultTransaction(client, payer), tx =>
        appendTransactionMessageInstruction(burnChecked, tx),
    );

    // Then it should fail with InsufficientFunds error.
    await t.throwsAsync(signAndSendTransaction(client, transactionMessage));
});

test('it fails when authority is not a signer', async t => {
    // Given a token account with tokens.
    const client = createDefaultSolanaClient();
    const [payer, mintAuthority, owner, wrongAuthority] = await Promise.all([
        generateKeyPairSignerWithSol(client),
        generateKeyPairSigner(),
        generateKeyPairSigner(),
        generateKeyPairSigner(),
    ]);
    const mint = await createMint(client, payer, mintAuthority.address, 9);
    const token = await createTokenWithAmount(client, payer, mintAuthority, mint, owner.address, 100n);

    // When we try to burn with wrong authority (not the owner).
    const burnChecked = getBurnCheckedInstruction({
        account: token,
        mint,
        authority: wrongAuthority, // Wrong authority!
        amount: 40n,
        decimals: 9,
    });
    const transactionMessage = await pipe(await createDefaultTransaction(client, payer), tx =>
        appendTransactionMessageInstruction(burnChecked, tx),
    );

    // Then it should fail (owner mismatch or missing signature).
    await t.throwsAsync(signAndSendTransaction(client, transactionMessage));
});

test('it fails when delegate has insufficient delegated amount', async t => {
    // Given a token account with 100 tokens and a delegate approved for only 20 tokens.
    const client = createDefaultSolanaClient();
    const [payer, mintAuthority, owner, delegate] = await Promise.all([
        generateKeyPairSignerWithSol(client),
        generateKeyPairSigner(),
        generateKeyPairSigner(),
        generateKeyPairSigner(),
    ]);
    const mint = await createMint(client, payer, mintAuthority.address, 9);
    const token = await createTokenWithAmount(client, payer, mintAuthority, mint, owner.address, 100n);

    // Approve delegate for only 20 tokens.
    const approve = getApproveInstruction({
        source: token,
        delegate: delegate.address,
        owner,
        amount: 20n,
    });
    await pipe(
        await createDefaultTransaction(client, payer),
        tx => appendTransactionMessageInstruction(approve, tx),
        tx => signAndSendTransaction(client, tx),
    );

    // When the delegate tries to burn 50 tokens (more than delegated).
    const burnChecked = getBurnCheckedInstruction({
        account: token,
        mint,
        authority: delegate,
        amount: 50n,
        decimals: 9,
    });
    const transactionMessage = await pipe(await createDefaultTransaction(client, payer), tx =>
        appendTransactionMessageInstruction(burnChecked, tx),
    );

    // Then it should fail with InsufficientFunds error.
    await t.throwsAsync(signAndSendTransaction(client, transactionMessage));
});

test('it burns zero tokens successfully', async t => {
    // Given a token account with tokens.
    const client = createDefaultSolanaClient();
    const [payer, mintAuthority, owner] = await Promise.all([
        generateKeyPairSignerWithSol(client),
        generateKeyPairSigner(),
        generateKeyPairSigner(),
    ]);
    const mint = await createMint(client, payer, mintAuthority.address, 9);
    const token = await createTokenWithAmount(client, payer, mintAuthority, mint, owner.address, 100n);

    // When we burn 0 tokens (edge case).
    const burnChecked = getBurnCheckedInstruction({
        account: token,
        mint,
        authority: owner,
        amount: 0n,
        decimals: 9,
    });
    await pipe(
        await createDefaultTransaction(client, payer),
        tx => appendTransactionMessageInstruction(burnChecked, tx),
        tx => signAndSendTransaction(client, tx),
    );

    // Then the balance should remain unchanged.
    const { data: tokenData } = await fetchToken(client.rpc, token);
    t.is(tokenData.amount, 100n);
});
