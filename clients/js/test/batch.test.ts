import { systemProgram } from '@solana-program/system';
import { AccountRole, generateKeyPairSigner, none, some } from '@solana/kit';
import { createLocalClient } from '@solana/kit-client-rpc';
import test from 'ava';
import {
    getBatchInstruction,
    getInitializeAccount3Instruction,
    getInitializeMint2Instruction,
    getMintSize,
    getMintToInstruction,
    parseBatchInstruction,
    TOKEN_PROGRAM_ADDRESS,
    TokenInstruction,
    tokenProgram,
} from '../src';

test('it batches multiple token instructions together', async t => {
    // Given a local validator client with some generated keypairs.
    const client = await createLocalClient().use(systemProgram()).use(tokenProgram());
    const [mint, token, mintAuthority, tokenOwner] = await Promise.all([
        generateKeyPairSigner(),
        generateKeyPairSigner(),
        generateKeyPairSigner(),
        generateKeyPairSigner(),
    ]);
    const mintSize = getMintSize();
    const tokenSize = getMintSize();

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
    t.like(mintAccount.data, {
        mintAuthority: some(mintAuthority.address),
        supply: 123_45n,
        decimals: 2,
        isInitialized: true,
        freezeAuthority: none(),
    });

    // And we expect the token account to have the correct data.
    const tokenAccount = await client.token.accounts.token.fetch(token.address);
    t.like(tokenAccount.data, {
        mint: mint.address,
        owner: tokenOwner.address,
        amount: 123_45n,
        isInitialized: true,
    });
});

test('it parses batch instructions including its inner instructions', async t => {
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
    t.deepEqual(parsedInstruction.instructions, [
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
