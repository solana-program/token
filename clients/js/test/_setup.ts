import path from 'node:path';

import { systemProgram } from '@solana-program/system';
import { Address, TransactionSigner, createClient, generateKeyPairSigner, lamports } from '@solana/kit';
import { litesvm } from '@solana/kit-plugin-litesvm';
import { airdropSigner, generatedSigner } from '@solana/kit-plugin-signer';

import {
    TOKEN_PROGRAM_ADDRESS,
    associatedTokenProgram,
    findAssociatedTokenPda,
    getTokenSize,
    tokenProgram,
} from '../src';

const TOKEN_BINARY_PATH = path.resolve(__dirname, '..', '..', '..', 'target', 'deploy', 'pinocchio_token_program.so');

export const createTestClient = () => {
    return createClient()
        .use(generatedSigner())
        .use(litesvm())
        .use(airdropSigner(lamports(1_000_000_000n)))
        .use(client => {
            // Load the token program into the LiteSVM instance from its compiled
            // `.so` file. This must run after the `litesvm()` plugin so that
            // `client.svm` is available. The system and associated-token
            // programs are LiteSVM builtins and need no loading.
            client.svm.addProgramFromFile(TOKEN_PROGRAM_ADDRESS, TOKEN_BINARY_PATH);
            return client;
        })
        .use(systemProgram())
        .use(tokenProgram())
        .use(associatedTokenProgram());
};

export type TestClient = Awaited<ReturnType<typeof createTestClient>>;

export const createToken = async (client: TestClient, mint: Address, owner: Address): Promise<Address> => {
    const space = BigInt(getTokenSize());
    const [rent, token] = await Promise.all([
        client.rpc.getMinimumBalanceForRentExemption(space).send(),
        generateKeyPairSigner(),
    ]);
    await client.sendTransaction([
        client.system.instructions.createAccount({
            newAccount: token,
            lamports: rent,
            space,
            programAddress: TOKEN_PROGRAM_ADDRESS,
        }),
        client.token.instructions.initializeAccount({ account: token.address, mint, owner }),
    ]);

    return token.address;
};

export const createTokenWithAmount = async (
    client: TestClient,
    mintAuthority: TransactionSigner,
    mint: Address,
    owner: Address,
    amount: bigint,
): Promise<Address> => {
    const space = BigInt(getTokenSize());
    const [rent, token] = await Promise.all([
        client.rpc.getMinimumBalanceForRentExemption(space).send(),
        generateKeyPairSigner(),
    ]);
    await client.sendTransaction([
        client.system.instructions.createAccount({
            newAccount: token,
            lamports: rent,
            space,
            programAddress: TOKEN_PROGRAM_ADDRESS,
        }),
        client.token.instructions.initializeAccount({ account: token.address, mint, owner }),
        client.token.instructions.mintTo({ mint, token: token.address, mintAuthority, amount }),
    ]);

    return token.address;
};

export const createTokenPdaWithAmount = async (
    client: TestClient,
    mintAuthority: TransactionSigner,
    mint: Address,
    owner: Address,
    amount: bigint,
    decimals: number,
): Promise<Address> => {
    await client.token.instructions.mintToATA({ owner, mint, mintAuthority, amount, decimals }).sendTransaction();
    const [token] = await findAssociatedTokenPda({ owner, mint, tokenProgram: TOKEN_PROGRAM_ADDRESS });
    return token;
};
