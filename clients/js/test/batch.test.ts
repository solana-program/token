import {
    AccountRole,
    decompileTransactionMessage,
    generateKeyPairSigner,
    getCompiledTransactionMessageDecoder,
    Instruction,
    InstructionWithData,
    none,
    ReadonlyUint8Array,
    some,
    Transaction,
} from '@solana/kit';
import { expect, it } from 'vitest';

import {
    AccountState,
    getBatchInstruction,
    getInitializeAccount3Instruction,
    getInitializeMint2Instruction,
    getMintSize,
    getMintToInstruction,
    getTokenSize,
    parseBatchInstruction,
    TOKEN_PROGRAM_ADDRESS,
    TokenInstruction,
} from '../src';
import { createTestClient } from './_setup';

it('batches multiple token instructions together', async () => {
    // Given a local validator client with some generated keypairs.
    const client = await createTestClient();
    const [mint, token, mintAuthority, tokenOwner] = await Promise.all([
        generateKeyPairSigner(),
        generateKeyPairSigner(),
        generateKeyPairSigner(),
        generateKeyPairSigner(),
    ]);
    const mintSize = getMintSize();
    const tokenSize = getTokenSize();

    // When we send a transaction with multiple token instructions batched together.
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

    // Then we expect the mint account to have the correct data.
    const mintAccount = await client.token.accounts.mint.fetch(mint.address);
    expect(mintAccount.data).toMatchObject({
        mintAuthority: some(mintAuthority.address),
        supply: 123_45n,
        decimals: 2,
        isInitialized: true,
        freezeAuthority: none(),
    });

    // And we expect the token account to have the correct data.
    const tokenAccount = await client.token.accounts.token.fetch(token.address);
    expect(tokenAccount.data).toMatchObject({
        mint: mint.address,
        owner: tokenOwner.address,
        amount: 123_45n,
        state: AccountState.Initialized,
    });
});

it('fails to batch nested batch instructions', async () => {
    // Given a local validator client with some generated keypairs.
    const client = await createTestClient();
    const [mint, token, mintAuthority, tokenOwner] = await Promise.all([
        generateKeyPairSigner(),
        generateKeyPairSigner(),
        generateKeyPairSigner(),
        generateKeyPairSigner(),
    ]);

    // When we try to create a batch instruction that contains another batch instruction as a child.
    const createNestedBatch = () =>
        client.token.instructions.batch([
            // @ts-expect-error - We expect a TypeScript error because batch instructions cannot be nested.
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
            ]),
            client.token.instructions.mintTo({
                mint: mint.address,
                token: token.address,
                mintAuthority: mintAuthority,
                amount: 123_45,
            }),
        ]);

    // Then we expect an error to be thrown.
    expect(createNestedBatch).toThrow('Batch instructions cannot be nested within other batch instructions.');
});

it('parses batch instructions including its inner instructions', async () => {
    // Given a batch instruction with multiple token inner instructions.
    const [mint, token, mintAuthority, tokenOwner] = await Promise.all([
        generateKeyPairSigner(),
        generateKeyPairSigner(),
        generateKeyPairSigner(),
        generateKeyPairSigner(),
    ]);
    const batchInstruction = getBatchInstruction([
        getInitializeMint2Instruction({
            mint: mint.address,
            decimals: 2,
            mintAuthority: mintAuthority.address,
        }),
        getInitializeAccount3Instruction({
            account: token.address,
            mint: mint.address,
            owner: tokenOwner.address,
        }),
        getMintToInstruction({
            mint: mint.address,
            token: token.address,
            mintAuthority: mintAuthority,
            amount: 123_45,
        }),
    ]);

    // When we parse the batch instruction.
    const parsedInstruction = parseBatchInstruction(batchInstruction);

    // Then we expect the parsed instruction to have the following inner instructions.
    expect(parsedInstruction.instructions).toEqual([
        {
            instructionType: TokenInstruction.InitializeMint2,
            programAddress: TOKEN_PROGRAM_ADDRESS,
            accounts: {
                mint: { address: mint.address, role: AccountRole.WRITABLE },
            },
            data: {
                decimals: 2,
                discriminator: 20,
                freezeAuthority: none(),
                mintAuthority: mintAuthority.address,
            },
        },
        {
            instructionType: TokenInstruction.InitializeAccount3,
            programAddress: TOKEN_PROGRAM_ADDRESS,
            accounts: {
                account: { address: token.address, role: AccountRole.WRITABLE },
                mint: { address: mint.address, role: AccountRole.READONLY },
            },
            data: {
                discriminator: 18,
                owner: tokenOwner.address,
            },
        },
        {
            instructionType: TokenInstruction.MintTo,
            programAddress: TOKEN_PROGRAM_ADDRESS,
            accounts: {
                mint: { address: mint.address, role: AccountRole.WRITABLE },
                mintAuthority: {
                    address: mintAuthority.address,
                    role: AccountRole.READONLY_SIGNER,
                    signer: mintAuthority,
                },
                token: { address: token.address, role: AccountRole.WRITABLE },
            },
            data: {
                amount: 123_45n,
                discriminator: 7,
            },
        },
    ]);
});

it('parses batch instructions from a fetched transaction', async () => {
    // Given a client with some generated keypairs.
    const client = await createTestClient();
    const [mint, token, mintAuthority, tokenOwner] = await Promise.all([
        generateKeyPairSigner(),
        generateKeyPairSigner(),
        generateKeyPairSigner(),
        generateKeyPairSigner(),
    ]);
    const mintSize = getMintSize();
    const tokenSize = getTokenSize();

    // And a sent transaction with a batch instruction.
    const result = await client.sendTransaction([
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

    // And given we access the batch instruction from the executed transaction,
    // which the transaction plan executor stores on the result context.
    const transaction = result.context.transaction as Transaction;
    expect(transaction).toBeTruthy();
    const compiledMessage = getCompiledTransactionMessageDecoder().decode(transaction.messageBytes);
    const message = decompileTransactionMessage(compiledMessage);
    const batchInstruction = message.instructions.find(
        instruction => instruction.programAddress === TOKEN_PROGRAM_ADDRESS,
    ) as Instruction & InstructionWithData<ReadonlyUint8Array>;

    // When we parse the batch instruction.
    const parsedInstruction = parseBatchInstruction(batchInstruction);

    // Then we expect the parsed instruction to have the following inner instructions.
    expect(parsedInstruction.instructions).toEqual([
        {
            instructionType: TokenInstruction.InitializeMint2,
            programAddress: TOKEN_PROGRAM_ADDRESS,
            accounts: {
                mint: { address: mint.address, role: AccountRole.WRITABLE_SIGNER },
            },
            data: {
                decimals: 2,
                discriminator: 20,
                freezeAuthority: none(),
                mintAuthority: mintAuthority.address,
            },
        },
        {
            instructionType: TokenInstruction.InitializeAccount3,
            programAddress: TOKEN_PROGRAM_ADDRESS,
            accounts: {
                account: { address: token.address, role: AccountRole.WRITABLE_SIGNER },
                mint: { address: mint.address, role: AccountRole.WRITABLE_SIGNER },
            },
            data: {
                discriminator: 18,
                owner: tokenOwner.address,
            },
        },
        {
            instructionType: TokenInstruction.MintTo,
            programAddress: TOKEN_PROGRAM_ADDRESS,
            accounts: {
                mint: { address: mint.address, role: AccountRole.WRITABLE_SIGNER },
                mintAuthority: { address: mintAuthority.address, role: AccountRole.READONLY_SIGNER },
                token: { address: token.address, role: AccountRole.WRITABLE_SIGNER },
            },
            data: {
                amount: 123_45n,
                discriminator: 7,
            },
        },
    ]);
});
