import {
    generateKeyPairSigner,
    Address,
    type SingleInstructionPlan,
    type SequentialInstructionPlan,
} from '@solana/kit';
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

// Extract the account addresses from a sequential instruction plan's instructions.
function getInstructionAccounts(plan: SequentialInstructionPlan): Address[][] {
    return plan.plans.map(p => {
        const single = p as SingleInstructionPlan;
        return (single.instruction.accounts ?? []).map((a: any) => a.address);
    });
}

test('it transfers tokens from one account to a new ATA', async t => {
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
    const tokenA = await createTokenWithAmount(client, payer, mintAuthority, mint, ownerA.address, 100n);

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
    const [{ data: mintData }, { data: tokenDataA }, { data: tokenDataB }] = await Promise.all([
        fetchMint(client.rpc, mint),
        fetchToken(client.rpc, tokenA),
        fetchToken(client.rpc, tokenB),
    ]);
    t.like(mintData, <Mint>{ supply: 100n });
    t.like(tokenDataA, <Token>{ amount: 50n });
    t.like(tokenDataB, <Token>{ amount: 50n });
});

test('derives a new ATA and transfers tokens to it', async t => {
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
    const tokenA = await createTokenWithAmount(client, payer, mintAuthority, mint, ownerA.address, 100n);

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

    const [{ data: mintData }, { data: tokenDataA }, { data: tokenDataB }] = await Promise.all([
        fetchMint(client.rpc, mint),
        fetchToken(client.rpc, tokenA),
        fetchToken(client.rpc, tokenB),
    ]);
    t.like(mintData, <Mint>{ supply: 100n });
    t.like(tokenDataA, <Token>{ amount: 50n });
    t.like(tokenDataB, <Token>{ amount: 50n });
});

test('it transfers tokens from one account to an existing ATA', async t => {
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
        createTokenWithAmount(client, payer, mintAuthority, mint, ownerA.address, 90n),
        createTokenPdaWithAmount(client, payer, mintAuthority, mint, ownerB.address, 10n, decimals),
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
    const [{ data: mintData }, { data: tokenDataA }, { data: tokenDataB }] = await Promise.all([
        fetchMint(client.rpc, mint),
        fetchToken(client.rpc, tokenA),
        fetchToken(client.rpc, tokenB),
    ]);
    t.like(mintData, <Mint>{ supply: 100n });
    t.like(tokenDataA, <Token>{ amount: 40n });
    t.like(tokenDataB, <Token>{ amount: 60n });
});

test('async variant auto-derives source ATA when omitted', async t => {
    // Given a keypair for the authority and a mint.
    const authority = await generateKeyPairSigner();
    const mint = (await generateKeyPairSigner()).address;
    const recipient = (await generateKeyPairSigner()).address;

    // Compute expected ATAs.
    const [expectedSource] = await findAssociatedTokenPda({
        owner: authority.address,
        mint,
        tokenProgram: TOKEN_PROGRAM_ADDRESS,
    });
    const [expectedDestination] = await findAssociatedTokenPda({
        owner: recipient,
        mint,
        tokenProgram: TOKEN_PROGRAM_ADDRESS,
    });

    // When building a plan WITHOUT source (should auto-derive).
    const plan = await getTransferToATAInstructionPlanAsync({
        payer: authority,
        mint,
        authority,
        recipient,
        amount: 100n,
        decimals: 9,
    });

    // Then the instruction plan should contain the correct derived addresses.
    const seqPlan = plan as SequentialInstructionPlan;
    t.is(seqPlan.kind, 'sequential');
    t.is(seqPlan.plans.length, 2);

    const accounts = getInstructionAccounts(seqPlan);

    // 1st instruction: createAssociatedTokenIdempotent — ata (index 1) should be the destination ATA
    t.is(accounts[0][1], expectedDestination);

    // 2nd instruction: transferChecked — source (index 0), destination (index 2)
    t.is(accounts[1][0], expectedSource);
    t.is(accounts[1][2], expectedDestination);
});

test('async variant auto-derives source from TransactionSigner authority', async t => {
    // Given a signer authority (has .address property).
    const authority = await generateKeyPairSigner();
    const mint = (await generateKeyPairSigner()).address;
    const recipient = (await generateKeyPairSigner()).address;

    const [expectedSource] = await findAssociatedTokenPda({
        owner: authority.address,
        mint,
        tokenProgram: TOKEN_PROGRAM_ADDRESS,
    });

    // When building a plan with authority as a TransactionSigner.
    const plan = await getTransferToATAInstructionPlanAsync({
        payer: authority,
        mint,
        authority,
        recipient,
        amount: 50n,
        decimals: 6,
    });

    // Then source should be derived from authority.address.
    const seqPlan = plan as SequentialInstructionPlan;
    const accounts = getInstructionAccounts(seqPlan);
    t.is(accounts[1][0], expectedSource);
});

test('async variant uses explicit source and destination when provided', async t => {
    // Given explicit source and destination addresses.
    const authority = await generateKeyPairSigner();
    const mint = (await generateKeyPairSigner()).address;
    const recipient = (await generateKeyPairSigner()).address;
    const explicitSource = (await generateKeyPairSigner()).address;
    const explicitDestination = (await generateKeyPairSigner()).address;

    // When building with explicit source and destination.
    const plan = await getTransferToATAInstructionPlanAsync({
        payer: authority,
        mint,
        authority,
        recipient,
        source: explicitSource,
        destination: explicitDestination,
        amount: 100n,
        decimals: 9,
    });

    // Then the plan should use the explicit addresses, not derived ones.
    const seqPlan = plan as SequentialInstructionPlan;
    const accounts = getInstructionAccounts(seqPlan);

    // createAssociatedTokenIdempotent — ata at index 1
    t.is(accounts[0][1], explicitDestination);

    // transferChecked — source at index 0, destination at index 2
    t.is(accounts[1][0], explicitSource);
    t.is(accounts[1][2], explicitDestination);
});
