import {
  Account,
  appendTransactionMessageInstruction,
  pipe,
} from '@solana/web3.js';
import test from 'ava';
import {
  Counter,
  fetchCounterFromSeeds,
  getCreateInstructionAsync,
} from '../src/index.js';
import {
  createDefaultSolanaClient,
  createDefaultTransaction,
  generateKeyPairSignerWithSol,
  signAndSendTransaction,
} from './_setup.js';

test('it creates a new counter account', async (t) => {
  // Given an authority key pair with some SOL.
  const client = createDefaultSolanaClient();
  const authority = await generateKeyPairSignerWithSol(client);

  // When we create a new counter account.
  const createIx = await getCreateInstructionAsync({ authority });
  await pipe(
    await createDefaultTransaction(client, authority),
    (tx) => appendTransactionMessageInstruction(createIx, tx),
    (tx) => signAndSendTransaction(client, tx)
  );

  // Then we expect the counter account to exist and have a value of 0.
  const counter = await fetchCounterFromSeeds(client.rpc, {
    authority: authority.address,
  });
  t.like(counter, <Account<Counter>>{
    data: {
      authority: authority.address,
      value: 0,
    },
  });
});
