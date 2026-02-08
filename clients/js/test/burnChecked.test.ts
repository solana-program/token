import {
  appendTransactionMessageInstruction,
  generateKeyPairSigner,
  pipe,
} from '@solana/kit';
import test from 'ava';
import {
  fetchMint,
  fetchToken,
  getApproveInstruction,
  getBurnCheckedInstruction,
} from '../src';
import {
  createDefaultSolanaClient,
  createDefaultTransaction,
  createMint,
  createTokenWithAmount,
  generateKeyPairSignerWithSol,
  signAndSendTransaction,
} from './_setup';

test('it burns tokens with correct decimals', async (t) => {
  // Given a mint with 9 decimals and a token account with 100 tokens.
  const client = createDefaultSolanaClient();
  const [payer, mintAuthority, owner] = await Promise.all([
    generateKeyPairSignerWithSol(client),
    generateKeyPairSigner(),
    generateKeyPairSigner(),
  ]);
  const mint = await createMint(client, payer, mintAuthority.address, 9);
  const token = await createTokenWithAmount(
    client,
    payer,
    mintAuthority,
    mint,
    owner.address,
    100n
  );

  // When we burn 40 tokens with the correct decimals (9).
  const burnChecked = getBurnCheckedInstruction({
    account: token,
    mint,
    authority: owner,
    amount: 40n,
    decimals: 9,
  });
  await pipe(
    await createDefaultTransaction(client, payer),
    (tx) => appendTransactionMessageInstruction(burnChecked, tx),
    (tx) => signAndSendTransaction(client, tx)
  );

  // Then we expect the token account to have 60 tokens remaining.
  const [{ data: mintData }, { data: tokenData }] = await Promise.all([
    fetchMint(client.rpc, mint),
    fetchToken(client.rpc, token),
  ]);
  t.is(tokenData.amount, 60n);
  t.is(mintData.supply, 60n);
});

test('it burns all tokens successfully', async (t) => {
  // Given a token account with 50 tokens.
  const client = createDefaultSolanaClient();
  const [payer, mintAuthority, owner] = await Promise.all([
    generateKeyPairSignerWithSol(client),
    generateKeyPairSigner(),
    generateKeyPairSigner(),
  ]);
  const mint = await createMint(client, payer, mintAuthority.address, 6);
  const token = await createTokenWithAmount(
    client,
    payer,
    mintAuthority,
    mint,
    owner.address,
    50n
  );

  // When we burn all 50 tokens.
  const burnChecked = getBurnCheckedInstruction({
    account: token,
    mint,
    authority: owner,
    amount: 50n,
    decimals: 6,
  });
  await pipe(
    await createDefaultTransaction(client, payer),
    (tx) => appendTransactionMessageInstruction(burnChecked, tx),
    (tx) => signAndSendTransaction(client, tx)
  );

  // Then the token account should have 0 tokens.
  const { data: tokenData } = await fetchToken(client.rpc, token);
  t.is(tokenData.amount, 0n);
});

test('it burns tokens using a delegate', async (t) => {
  // Given a token account with 100 tokens and a delegate approved for 60 tokens.
  const client = createDefaultSolanaClient();
  const [payer, mintAuthority, owner, delegate] = await Promise.all([
    generateKeyPairSignerWithSol(client),
    generateKeyPairSigner(),
    generateKeyPairSigner(),
    generateKeyPairSigner(),
  ]);
  const mint = await createMint(client, payer, mintAuthority.address, 9);
  const token = await createTokenWithAmount(
    client,
    payer,
    mintAuthority,
    mint,
    owner.address,
    100n
  );

  // Approve delegate for 60 tokens.
  const approve = getApproveInstruction({
    source: token,
    delegate: delegate.address,
    owner,
    amount: 60n,
  });
  await pipe(
    await createDefaultTransaction(client, payer),
    (tx) => appendTransactionMessageInstruction(approve, tx),
    (tx) => signAndSendTransaction(client, tx)
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
    (tx) => appendTransactionMessageInstruction(burnChecked, tx),
    (tx) => signAndSendTransaction(client, tx)
  );

  // Then the token account should have 70 tokens remaining.
  const { data: tokenData } = await fetchToken(client.rpc, token);
  t.is(tokenData.amount, 70n);
  t.is(tokenData.delegatedAmount, 30n); // Remaining delegated amount
});

test('it updates mint supply correctly after burn', async (t) => {
  // Given a mint with total supply of 200 tokens across multiple accounts.
  const client = createDefaultSolanaClient();
  const [payer, mintAuthority, ownerA, ownerB] = await Promise.all([
    generateKeyPairSignerWithSol(client),
    generateKeyPairSigner(),
    generateKeyPairSigner(),
    generateKeyPairSigner(),
  ]);
  const mint = await createMint(client, payer, mintAuthority.address, 0);
  const [tokenA] = await Promise.all([
    createTokenWithAmount(
      client,
      payer,
      mintAuthority,
      mint,
      ownerA.address,
      100n
    ),
    createTokenWithAmount(
      client,
      payer,
      mintAuthority,
      mint,
      ownerB.address,
      100n
    ),
  ]);

  // When owner A burns 25 tokens.
  const burnChecked = getBurnCheckedInstruction({
    account: tokenA,
    mint,
    authority: ownerA,
    amount: 25n,
    decimals: 0,
  });
  await pipe(
    await createDefaultTransaction(client, payer),
    (tx) => appendTransactionMessageInstruction(burnChecked, tx),
    (tx) => signAndSendTransaction(client, tx)
  );

  // Then the total supply should decrease to 175.
  const { data: mintData } = await fetchMint(client.rpc, mint);
  t.is(mintData.supply, 175n);
});

test('it fails when decimals mismatch', async (t) => {
  // Given a mint with 9 decimals and a token account with tokens.
  const client = createDefaultSolanaClient();
  const [payer, mintAuthority, owner] = await Promise.all([
    generateKeyPairSignerWithSol(client),
    generateKeyPairSigner(),
    generateKeyPairSigner(),
  ]);
  const mint = await createMint(client, payer, mintAuthority.address, 9);
  const token = await createTokenWithAmount(
    client,
    payer,
    mintAuthority,
    mint,
    owner.address,
    100n
  );

  // When we try to burn with incorrect decimals (6 instead of 9).
  const burnChecked = getBurnCheckedInstruction({
    account: token,
    mint,
    authority: owner,
    amount: 40n,
    decimals: 6, // Wrong! Should be 9
  });
  const transactionMessage = await pipe(
    await createDefaultTransaction(client, payer),
    (tx) => appendTransactionMessageInstruction(burnChecked, tx)
  );

  // Then it should fail with MintDecimalsMismatch error.
  await t.throwsAsync(signAndSendTransaction(client, transactionMessage));
});

test('it fails when burning more than account balance', async (t) => {
  // Given a token account with only 50 tokens.
  const client = createDefaultSolanaClient();
  const [payer, mintAuthority, owner] = await Promise.all([
    generateKeyPairSignerWithSol(client),
    generateKeyPairSigner(),
    generateKeyPairSigner(),
  ]);
  const mint = await createMint(client, payer, mintAuthority.address, 9);
  const token = await createTokenWithAmount(
    client,
    payer,
    mintAuthority,
    mint,
    owner.address,
    50n
  );

  // When we try to burn 150 tokens (more than available).
  const burnChecked = getBurnCheckedInstruction({
    account: token,
    mint,
    authority: owner,
    amount: 150n,
    decimals: 9,
  });
  const transactionMessage = await pipe(
    await createDefaultTransaction(client, payer),
    (tx) => appendTransactionMessageInstruction(burnChecked, tx)
  );

  // Then it should fail with InsufficientFunds error.
  await t.throwsAsync(signAndSendTransaction(client, transactionMessage));
});

test('it fails when authority is not a signer', async (t) => {
  // Given a token account with tokens.
  const client = createDefaultSolanaClient();
  const [payer, mintAuthority, owner, wrongAuthority] = await Promise.all([
    generateKeyPairSignerWithSol(client),
    generateKeyPairSigner(),
    generateKeyPairSigner(),
    generateKeyPairSigner(),
  ]);
  const mint = await createMint(client, payer, mintAuthority.address, 9);
  const token = await createTokenWithAmount(
    client,
    payer,
    mintAuthority,
    mint,
    owner.address,
    100n
  );

  // When we try to burn with wrong authority (not the owner).
  const burnChecked = getBurnCheckedInstruction({
    account: token,
    mint,
    authority: wrongAuthority, // Wrong authority!
    amount: 40n,
    decimals: 9,
  });
  const transactionMessage = await pipe(
    await createDefaultTransaction(client, payer),
    (tx) => appendTransactionMessageInstruction(burnChecked, tx)
  );

  // Then it should fail (owner mismatch or missing signature).
  await t.throwsAsync(signAndSendTransaction(client, transactionMessage));
});

test('it fails when delegate has insufficient delegated amount', async (t) => {
  // Given a token account with 100 tokens and a delegate approved for only 20 tokens.
  const client = createDefaultSolanaClient();
  const [payer, mintAuthority, owner, delegate] = await Promise.all([
    generateKeyPairSignerWithSol(client),
    generateKeyPairSigner(),
    generateKeyPairSigner(),
    generateKeyPairSigner(),
  ]);
  const mint = await createMint(client, payer, mintAuthority.address, 9);
  const token = await createTokenWithAmount(
    client,
    payer,
    mintAuthority,
    mint,
    owner.address,
    100n
  );

  // Approve delegate for only 20 tokens.
  const approve = getApproveInstruction({
    source: token,
    delegate: delegate.address,
    owner,
    amount: 20n,
  });
  await pipe(
    await createDefaultTransaction(client, payer),
    (tx) => appendTransactionMessageInstruction(approve, tx),
    (tx) => signAndSendTransaction(client, tx)
  );

  // When the delegate tries to burn 50 tokens (more than delegated).
  const burnChecked = getBurnCheckedInstruction({
    account: token,
    mint,
    authority: delegate,
    amount: 50n,
    decimals: 9,
  });
  const transactionMessage = await pipe(
    await createDefaultTransaction(client, payer),
    (tx) => appendTransactionMessageInstruction(burnChecked, tx)
  );

  // Then it should fail with InsufficientFunds error.
  await t.throwsAsync(signAndSendTransaction(client, transactionMessage));
});

test('it burns zero tokens successfully', async (t) => {
  // Given a token account with tokens.
  const client = createDefaultSolanaClient();
  const [payer, mintAuthority, owner] = await Promise.all([
    generateKeyPairSignerWithSol(client),
    generateKeyPairSigner(),
    generateKeyPairSigner(),
  ]);
  const mint = await createMint(client, payer, mintAuthority.address, 9);
  const token = await createTokenWithAmount(
    client,
    payer,
    mintAuthority,
    mint,
    owner.address,
    100n
  );

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
    (tx) => appendTransactionMessageInstruction(burnChecked, tx),
    (tx) => signAndSendTransaction(client, tx)
  );

  // Then the balance should remain unchanged.
  const { data: tokenData } = await fetchToken(client.rpc, token);
  t.is(tokenData.amount, 100n);
});

test('it burns with different decimal precisions', async (t) => {
  // Given mints with different decimal precisions.
  const client = createDefaultSolanaClient();
  const [payer, mintAuthority, owner] = await Promise.all([
    generateKeyPairSignerWithSol(client),
    generateKeyPairSigner(),
    generateKeyPairSigner(),
  ]);

  // Test with 0 decimals (integer tokens).
  const mint0 = await createMint(client, payer, mintAuthority.address, 0);
  const token0 = await createTokenWithAmount(
    client,
    payer,
    mintAuthority,
    mint0,
    owner.address,
    100n
  );

  const burnChecked0 = getBurnCheckedInstruction({
    account: token0,
    mint: mint0,
    authority: owner,
    amount: 25n,
    decimals: 0,
  });
  await pipe(
    await createDefaultTransaction(client, payer),
    (tx) => appendTransactionMessageInstruction(burnChecked0, tx),
    (tx) => signAndSendTransaction(client, tx)
  );

  const { data: tokenData0 } = await fetchToken(client.rpc, token0);
  t.is(tokenData0.amount, 75n);

  // Test with 18 decimals (maximum precision).
  const mint18 = await createMint(client, payer, mintAuthority.address, 18);
  const token18 = await createTokenWithAmount(
    client,
    payer,
    mintAuthority,
    mint18,
    owner.address,
    1000000000000000000n // 1 token with 18 decimals
  );

  const burnChecked18 = getBurnCheckedInstruction({
    account: token18,
    mint: mint18,
    authority: owner,
    amount: 250000000000000000n, // 0.25 tokens
    decimals: 18,
  });
  await pipe(
    await createDefaultTransaction(client, payer),
    (tx) => appendTransactionMessageInstruction(burnChecked18, tx),
    (tx) => signAndSendTransaction(client, tx)
  );

  const { data: tokenData18 } = await fetchToken(client.rpc, token18);
  t.is(tokenData18.amount, 750000000000000000n); // 0.75 tokens remaining
});
