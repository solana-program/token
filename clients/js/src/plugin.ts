import { ClientWithPayer, pipe } from '@solana/kit';
import { addSelfPlanAndSendFunctions, SelfPlanAndSendFunctions } from '@solana/kit/program-client-core';

import { CreateMintInstructionPlanInput, getCreateMintInstructionPlan } from './createMint';
import {
    TokenPlugin as GeneratedTokenPlugin,
    TokenPluginInstructions as GeneratedTokenPluginInstructions,
    TokenPluginRequirements as GeneratedTokenPluginRequirements,
    tokenProgram as generatedTokenProgram,
} from './generated';
import { getMintToATAInstructionPlanAsync, MintToATAInstructionPlanAsyncInput } from './mintToATA';
import { getTransferToATAInstructionPlanAsync, TransferToATAInstructionPlanAsyncInput } from './transferToATA';

export type TokenPluginRequirements = GeneratedTokenPluginRequirements & ClientWithPayer;

export type TokenPlugin = Omit<GeneratedTokenPlugin, 'instructions'> & { instructions: TokenPluginInstructions };

export type TokenPluginInstructions = GeneratedTokenPluginInstructions & {
    /** Create a new token mint. */
    createMint: (
        input: MakeOptional<CreateMintInstructionPlanInput, 'payer' | 'mintAuthority'>,
    ) => ReturnType<typeof getCreateMintInstructionPlan> & SelfPlanAndSendFunctions;
    /** Mint tokens to an owner's ATA (created if needed). */
    mintToATA: (
        input: MakeOptional<MintToATAInstructionPlanAsyncInput, 'payer' | 'mintAuthority'>,
    ) => Promise<Awaited<ReturnType<typeof getMintToATAInstructionPlanAsync>>> & SelfPlanAndSendFunctions;
    /** Transfer tokens to a recipient's ATA (created if needed). */
    transferToATA: (
        input: MakeOptional<TransferToATAInstructionPlanAsyncInput, 'payer' | 'authority'>,
    ) => Promise<Awaited<ReturnType<typeof getTransferToATAInstructionPlanAsync>>> & SelfPlanAndSendFunctions;
};

export function tokenProgram() {
    return <T extends TokenPluginRequirements>(client: T) => {
        return pipe(client, generatedTokenProgram(), c => ({
            ...c,
            token: <TokenPlugin>{
                ...c.token,
                instructions: {
                    ...c.token.instructions,
                    createMint: input =>
                        addSelfPlanAndSendFunctions(
                            client,
                            getCreateMintInstructionPlan({
                                ...input,
                                payer: input.payer ?? client.payer,
                                mintAuthority: input.mintAuthority ?? client.payer.address,
                            }),
                        ),
                    mintToATA: input =>
                        addSelfPlanAndSendFunctions(
                            client,
                            getMintToATAInstructionPlanAsync({
                                ...input,
                                payer: input.payer ?? client.payer,
                                mintAuthority: input.mintAuthority ?? client.payer,
                            }),
                        ),
                    transferToATA: input =>
                        addSelfPlanAndSendFunctions(
                            client,
                            getTransferToATAInstructionPlanAsync({
                                ...input,
                                payer: input.payer ?? client.payer,
                                authority: input.authority ?? client.payer,
                            }),
                        ),
                },
            },
        }));
    };
}

type MakeOptional<T, K extends keyof T> = Omit<T, K> & Partial<Pick<T, K>>;
