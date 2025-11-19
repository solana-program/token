import { Account, generateKeyPairSigner, none } from '@solana/kit';
import test from 'ava';
import {
  AccountState,
  TOKEN_PROGRAM_ADDRESS,
  Token,
  getMintToATAInstructionPlan,
  getMintToATAInstructionPlanAsync,
  fetchToken,
  findAssociatedTokenPda,
} from '../src';
import {
  createDefaultSolanaClient,
  createDefaultTransactionPlanner,
  createMint,
  generateKeyPairSignerWithSol,
} from './_setup';

test('it creates a new associated token account with an initial balance', async (t) => {
  // Given a mint account, its mint authority, a token owner and the ATA.
  const client = createDefaultSolanaClient();
  const [payer, mintAuthority, owner] = await Promise.all([
    generateKeyPairSignerWithSol(client),
    generateKeyPairSigner(),
    generateKeyPairSigner(),
  ]);
  const decimals = 2;
  const mint = await createMint(client, payer, mintAuthority.address, decimals);
  const [ata] = await findAssociatedTokenPda({
    mint,
    owner: owner.address,
    tokenProgram: TOKEN_PROGRAM_ADDRESS,
  });

  // When we mint to a token account at this address.
  const instructionPlan = getMintToATAInstructionPlan({
    payer,
    ata,
    mint,
    owner: owner.address,
    mintAuthority,
    amount: 1_000n,
    decimals,
  });

  const transactionPlanner = createDefaultTransactionPlanner(client, payer);
  const transactionPlan = await transactionPlanner(instructionPlan);
  await client.sendTransactionPlan(transactionPlan);

  // Then we expect the token account to exist and have the following data.
  t.like(await fetchToken(client.rpc, ata), <Account<Token>>{
    address: ata,
    data: {
      mint,
      owner: owner.address,
      amount: 1000n,
      delegate: none(),
      state: AccountState.Initialized,
      isNative: none(),
      delegatedAmount: 0n,
      closeAuthority: none(),
    },
  });
});

test('it derives a new associated token account with an initial balance', async (t) => {
  // Given a mint account, its mint authority, a token owner and the ATA.
  const client = createDefaultSolanaClient();
  const [payer, mintAuthority, owner] = await Promise.all([
    generateKeyPairSignerWithSol(client),
    generateKeyPairSigner(),
    generateKeyPairSigner(),
  ]);
  const decimals = 2;
  const mint = await createMint(client, payer, mintAuthority.address, decimals);

  // When we mint to a token account for the mint.
  const instructionPlan = await getMintToATAInstructionPlanAsync({
    payer,
    mint,
    owner: owner.address,
    mintAuthority,
    amount: 1_000n,
    decimals,
  });

  const transactionPlanner = createDefaultTransactionPlanner(client, payer);
  const transactionPlan = await transactionPlanner(instructionPlan);
  await client.sendTransactionPlan(transactionPlan);

  // Then we expect the token account to exist and have the following data.
  const [ata] = await findAssociatedTokenPda({
    mint,
    owner: owner.address,
    tokenProgram: TOKEN_PROGRAM_ADDRESS,
  });

  t.like(await fetchToken(client.rpc, ata), <Account<Token>>{
    address: ata,
    data: {
      mint,
      owner: owner.address,
      amount: 1000n,
      delegate: none(),
      state: AccountState.Initialized,
      isNative: none(),
      delegatedAmount: 0n,
      closeAuthority: none(),
    },
  });
});

test('it also mints to an existing associated token account', async (t) => {
  // Given a mint account, its mint authority, a token owner and the ATA.
  const client = createDefaultSolanaClient();
  const [payer, mintAuthority, owner] = await Promise.all([
    generateKeyPairSignerWithSol(client),
    generateKeyPairSigner(),
    generateKeyPairSigner(),
  ]);
  const decimals = 2;
  const mint = await createMint(client, payer, mintAuthority.address, decimals);
  const [ata] = await findAssociatedTokenPda({
    mint,
    owner: owner.address,
    tokenProgram: TOKEN_PROGRAM_ADDRESS,
  });

  // When we create and initialize a token account at this address.
  const instructionPlan = getMintToATAInstructionPlan({
    payer,
    ata,
    mint,
    owner: owner.address,
    mintAuthority,
    amount: 1_000n,
    decimals,
  });

  const transactionPlanner = createDefaultTransactionPlanner(client, payer);
  const transactionPlan = await transactionPlanner(instructionPlan);
  await client.sendTransactionPlan(transactionPlan);

  // And then we mint additional tokens to the same account.
  const instructionPlan2 = getMintToATAInstructionPlan({
    payer,
    ata,
    mint,
    owner: owner.address,
    mintAuthority,
    amount: 1_000n,
    decimals,
  });

  const transactionPlan2 = await transactionPlanner(instructionPlan2);
  await client.sendTransactionPlan(transactionPlan2);

  // Then we expect the token account to exist and have the following data.
  t.like(await fetchToken(client.rpc, ata), <Account<Token>>{
    address: ata,
    data: {
      mint,
      owner: owner.address,
      amount: 2000n,
      delegate: none(),
      state: AccountState.Initialized,
      isNative: none(),
      delegatedAmount: 0n,
      closeAuthority: none(),
    },
  });
});
