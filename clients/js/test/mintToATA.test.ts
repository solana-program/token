import {
    Account,
    Address,
    generateKeyPairSigner,
    none,
    type SingleInstructionPlan,
    type SequentialInstructionPlan,
} from '@solana/kit';
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

// Extract the account addresses from a sequential instruction plan's instructions.
function getInstructionAccounts(plan: SequentialInstructionPlan): Address[][] {
    return plan.plans.map(p => {
        const single = p as SingleInstructionPlan;
        return (single.instruction.accounts ?? []).map((a: any) => a.address);
    });
}

test('it creates a new associated token account with an initial balance', async t => {
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

test('it derives a new associated token account with an initial balance', async t => {
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

test('it also mints to an existing associated token account', async t => {
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

// --- Offline tests: verify derived addresses in instruction plans ---

test('async variant auto-derives ATA from owner + mint', async t => {
    // Given an owner and mint.
    const payer = await generateKeyPairSigner();
    const mintAuthority = await generateKeyPairSigner();
    const owner = (await generateKeyPairSigner()).address;
    const mint = (await generateKeyPairSigner()).address;

    const [expectedAta] = await findAssociatedTokenPda({
        owner,
        mint,
        tokenProgram: TOKEN_PROGRAM_ADDRESS,
    });

    // When building a plan without an explicit ATA.
    const plan = await getMintToATAInstructionPlanAsync({
        payer,
        mint,
        owner,
        mintAuthority,
        amount: 500n,
        decimals: 6,
    });

    // Then the plan should contain the derived ATA.
    const seqPlan = plan as SequentialInstructionPlan;
    t.is(seqPlan.kind, 'sequential');
    t.is(seqPlan.plans.length, 2);

    const accounts = getInstructionAccounts(seqPlan);

    // createAssociatedTokenIdempotent — ata at index 1
    t.is(accounts[0][1], expectedAta);

    // mintToChecked — token at index 1
    t.is(accounts[1][1], expectedAta);
});

test('async variant uses explicit ATA when provided', async t => {
    // Given an explicit ATA address.
    const payer = await generateKeyPairSigner();
    const mintAuthority = await generateKeyPairSigner();
    const owner = (await generateKeyPairSigner()).address;
    const mint = (await generateKeyPairSigner()).address;
    const explicitAta = (await generateKeyPairSigner()).address;

    // When building a plan with the explicit ATA.
    const plan = await getMintToATAInstructionPlanAsync({
        payer,
        mint,
        owner,
        mintAuthority,
        ata: explicitAta,
        amount: 500n,
        decimals: 6,
    });

    // Then the plan should use the explicit ATA, not a derived one.
    const seqPlan = plan as SequentialInstructionPlan;
    const accounts = getInstructionAccounts(seqPlan);

    t.is(accounts[0][1], explicitAta);
    t.is(accounts[1][1], explicitAta);
});
