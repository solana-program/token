import { generateKeyPairSigner } from '@solana/kit';
import test from 'ava';
import {
  Mint,
  TOKEN_PROGRAM_ADDRESS,
  Token,
  fetchMint,
  fetchToken,
  findAssociatedTokenPda,
  getTransferToATAInstructionPlan,
  getTransferToATAInstructionPlanAsync,
} from '../src';
import {
  createDefaultSolanaClient,
  createDefaultTransactionPlanner,
  createMint,
  createTokenPdaWithAmount,
  createTokenWithAmount,
  generateKeyPairSignerWithSol,
} from './_setup';

test('it transfers tokens from one account to a new ATA', async (t) => {
  // Given a mint account, one token account with 100 tokens, and a second owner.
  const client = createDefaultSolanaClient();
  const [payer, mintAuthority, ownerA, ownerB] = await Promise.all([
    generateKeyPairSignerWithSol(client),
    generateKeyPairSigner(),
    generateKeyPairSigner(),
    generateKeyPairSigner(),
  ]);
  const decimals = 2;
  const mint = await createMint(client, payer, mintAuthority.address, decimals);
  const tokenA = await createTokenWithAmount(
    client,
    payer,
    mintAuthority,
    mint,
    ownerA.address,
    100n
  );

  const [tokenB] = await findAssociatedTokenPda({
    owner: ownerB.address,
    mint,
    tokenProgram: TOKEN_PROGRAM_ADDRESS,
  });

  // When owner A transfers 50 tokens to owner B.
  const instructionPlan = getTransferToATAInstructionPlan({
    payer,
    mint,
    source: tokenA,
    authority: ownerA,
    destination: tokenB,
    recipient: ownerB.address,
    amount: 50n,
    decimals,
  });

  const transactionPlanner = createDefaultTransactionPlanner(client, payer);
  const transactionPlan = await transactionPlanner(instructionPlan);
  await client.sendTransactionPlan(transactionPlan);

  // Then we expect the mint and token accounts to have the following updated data.
  const [{ data: mintData }, { data: tokenDataA }, { data: tokenDataB }] =
    await Promise.all([
      fetchMint(client.rpc, mint),
      fetchToken(client.rpc, tokenA),
      fetchToken(client.rpc, tokenB),
    ]);
  t.like(mintData, <Mint>{ supply: 100n });
  t.like(tokenDataA, <Token>{ amount: 50n });
  t.like(tokenDataB, <Token>{ amount: 50n });
});

test('derives a new ATA and transfers tokens to it', async (t) => {
  // Given a mint account, one token account with 100 tokens, and a second owner.
  const client = createDefaultSolanaClient();
  const [payer, mintAuthority, ownerA, ownerB] = await Promise.all([
    generateKeyPairSignerWithSol(client),
    generateKeyPairSigner(),
    generateKeyPairSigner(),
    generateKeyPairSigner(),
  ]);
  const decimals = 2;
  const mint = await createMint(client, payer, mintAuthority.address, decimals);
  const tokenA = await createTokenWithAmount(
    client,
    payer,
    mintAuthority,
    mint,
    ownerA.address,
    100n
  );

  // When owner A transfers 50 tokens to owner B.
  const instructionPlan = await getTransferToATAInstructionPlanAsync({
    payer,
    mint,
    source: tokenA,
    authority: ownerA,
    recipient: ownerB.address,
    amount: 50n,
    decimals,
  });

  const transactionPlanner = createDefaultTransactionPlanner(client, payer);
  const transactionPlan = await transactionPlanner(instructionPlan);
  await client.sendTransactionPlan(transactionPlan);

  // Then we expect the mint and token accounts to have the following updated data.
  const [tokenB] = await findAssociatedTokenPda({
    owner: ownerB.address,
    mint,
    tokenProgram: TOKEN_PROGRAM_ADDRESS,
  });

  const [{ data: mintData }, { data: tokenDataA }, { data: tokenDataB }] =
    await Promise.all([
      fetchMint(client.rpc, mint),
      fetchToken(client.rpc, tokenA),
      fetchToken(client.rpc, tokenB),
    ]);
  t.like(mintData, <Mint>{ supply: 100n });
  t.like(tokenDataA, <Token>{ amount: 50n });
  t.like(tokenDataB, <Token>{ amount: 50n });
});

test('it transfers tokens from one account to an existing ATA', async (t) => {
  // Given a mint account and two token accounts.
  // One with 90 tokens and the other with 10 tokens.
  const client = createDefaultSolanaClient();
  const [payer, mintAuthority, ownerA, ownerB] = await Promise.all([
    generateKeyPairSignerWithSol(client),
    generateKeyPairSigner(),
    generateKeyPairSigner(),
    generateKeyPairSigner(),
  ]);
  const decimals = 2;
  const mint = await createMint(client, payer, mintAuthority.address, decimals);
  const [tokenA, tokenB] = await Promise.all([
    createTokenWithAmount(
      client,
      payer,
      mintAuthority,
      mint,
      ownerA.address,
      90n
    ),
    createTokenPdaWithAmount(
      client,
      payer,
      mintAuthority,
      mint,
      ownerB.address,
      10n,
      decimals
    ),
  ]);

  // When owner A transfers 50 tokens to owner B.
  const instructionPlan = getTransferToATAInstructionPlan({
    payer,
    mint,
    source: tokenA,
    authority: ownerA,
    destination: tokenB,
    recipient: ownerB.address,
    amount: 50n,
    decimals,
  });

  const transactionPlanner = createDefaultTransactionPlanner(client, payer);
  const transactionPlan = await transactionPlanner(instructionPlan);
  await client.sendTransactionPlan(transactionPlan);

  // Then we expect the mint and token accounts to have the following updated data.
  const [{ data: mintData }, { data: tokenDataA }, { data: tokenDataB }] =
    await Promise.all([
      fetchMint(client.rpc, mint),
      fetchToken(client.rpc, tokenA),
      fetchToken(client.rpc, tokenB),
    ]);
  t.like(mintData, <Mint>{ supply: 100n });
  t.like(tokenDataA, <Token>{ amount: 40n });
  t.like(tokenDataB, <Token>{ amount: 60n });
});
