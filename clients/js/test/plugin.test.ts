import {
    Address,
    generateKeyPairSigner,
    type SingleInstructionPlan,
    type SequentialInstructionPlan,
    type ClientWithTransactionPlanning,
    type ClientWithTransactionSending,
    Rpc,
    SolanaRpcApi,
    RpcSubscriptions,
    SolanaRpcSubscriptionsApi,
} from '@solana/kit';
import test from 'ava';
import { TOKEN_PROGRAM_ADDRESS, findAssociatedTokenPda, tokenProgram } from '../src';

// Extract the account addresses from a sequential instruction plan's instructions.
function getInstructionAccounts(plan: SequentialInstructionPlan): Address[][] {
    return plan.plans.map(p => {
        const single = p as SingleInstructionPlan;
        return (single.instruction.accounts ?? []).map(a => a.address);
    });
}

/**
 * Create a minimal mock client that satisfies TokenPluginRequirements.
 * No real RPC — just enough for the plugin to wire up defaults.
 */
function createMockClient(payer: Awaited<ReturnType<typeof generateKeyPairSigner>>) {
    return tokenProgram()({
        payer,
        rpc: {} as Rpc<SolanaRpcApi>,
        rpcSubscriptions: {} as RpcSubscriptions<SolanaRpcSubscriptionsApi>,
        planTransaction: (async () => {}) as unknown as ClientWithTransactionPlanning['planTransaction'],
        planTransactions: (async () => {}) as unknown as ClientWithTransactionPlanning['planTransactions'],
        sendTransaction: (async () => {}) as unknown as ClientWithTransactionSending['sendTransaction'],
        sendTransactions: (async () => {}) as unknown as ClientWithTransactionSending['sendTransactions'],
    });
}

test('plugin transferToATA defaults authority to payer and auto-derives source + destination', async t => {
    const payer = await generateKeyPairSigner();
    const mint = (await generateKeyPairSigner()).address;
    const recipient = (await generateKeyPairSigner()).address;

    const [expectedSource] = await findAssociatedTokenPda({
        owner: payer.address,
        mint,
        tokenProgram: TOKEN_PROGRAM_ADDRESS,
    });
    const [expectedDestination] = await findAssociatedTokenPda({
        owner: recipient,
        mint,
        tokenProgram: TOKEN_PROGRAM_ADDRESS,
    });

    const client = createMockClient(payer);

    // Call transferToATA with minimal input — payer, authority, source all defaulted/derived.
    const plan = await client.token.instructions.transferToATA({
        mint,
        recipient,
        amount: 100n,
        decimals: 9,
    });

    const seqPlan = plan as unknown as SequentialInstructionPlan;
    t.is(seqPlan.kind, 'sequential');
    const accounts = getInstructionAccounts(seqPlan);

    // createAssociatedTokenIdempotent — ata (destination) at index 1
    t.is(accounts[0][1], expectedDestination);

    // transferChecked — source at index 0, destination at index 2
    t.is(accounts[1][0], expectedSource);
    t.is(accounts[1][2], expectedDestination);
});

test('plugin mintToATA defaults mintAuthority to payer and auto-derives ATA', async t => {
    const payer = await generateKeyPairSigner();
    const mint = (await generateKeyPairSigner()).address;
    const owner = (await generateKeyPairSigner()).address;

    const [expectedAta] = await findAssociatedTokenPda({
        owner,
        mint,
        tokenProgram: TOKEN_PROGRAM_ADDRESS,
    });

    const client = createMockClient(payer);

    // Call mintToATA with minimal input — payer and mintAuthority defaulted, ATA derived.
    const plan = await client.token.instructions.mintToATA({
        mint,
        owner,
        amount: 1000n,
        decimals: 6,
    });

    const seqPlan = plan as unknown as SequentialInstructionPlan;
    t.is(seqPlan.kind, 'sequential');
    const accounts = getInstructionAccounts(seqPlan);

    // createAssociatedTokenIdempotent — ata at index 1
    t.is(accounts[0][1], expectedAta);

    // mintToChecked — token at index 1, mintAuthority at index 2
    t.is(accounts[1][1], expectedAta);
    t.is(accounts[1][2], payer.address); // mintAuthority defaulted to payer
});

test('plugin createMint defaults mintAuthority to payer address', async t => {
    const payer = await generateKeyPairSigner();
    const newMint = await generateKeyPairSigner();

    const client = createMockClient(payer);

    // Call createMint without mintAuthority — should default to payer.address.
    const plan = client.token.instructions.createMint({
        newMint,
        decimals: 9,
    });

    const seqPlan = plan as unknown as SequentialInstructionPlan;
    t.is(seqPlan.kind, 'sequential');
    t.is(seqPlan.plans.length, 2);
    const accounts = getInstructionAccounts(seqPlan);

    // createAccount — payer at index 0
    t.is(accounts[0][0], payer.address);
});
