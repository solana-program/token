import { ClientWithPayer, pipe } from '@solana/kit';
import { addSelfPlanAndSendFunctions, SelfPlanAndSendFunctions } from '@solana/kit/program-client-core';

import { getBatchInstruction } from './batch';
import {
    CreateMintInstructionPlanConfig,
    CreateMintInstructionPlanInput,
    getCreateMintInstructionPlan,
} from './createMint';
import {
    TokenPlugin as GeneratedTokenPlugin,
    TokenPluginInstructions as GeneratedTokenPluginInstructions,
    TokenPluginRequirements as GeneratedTokenPluginRequirements,
    tokenProgram as generatedTokenProgram,
} from './generated';
import {
    getMintToATAInstructionPlanAsync,
    MintToATAInstructionPlanAsyncInput,
    MintToATAInstructionPlanConfig,
} from './mintToATA';
import {
    getTransferToATAInstructionPlanAsync,
    TransferToATAInstructionPlanAsyncInput,
    TransferToATAInstructionPlanConfig,
} from './transferToATA';
import { MakeOptional } from './types';

export type TokenPluginRequirements = GeneratedTokenPluginRequirements & ClientWithPayer;

export type TokenPlugin = Omit<GeneratedTokenPlugin, 'instructions'> & { instructions: TokenPluginInstructions };

export type TokenPluginInstructions = Omit<GeneratedTokenPluginInstructions, 'batch'> & {
    /** Batch multiple instructions into one by using other token instructions as children. */
    batch: (
        instructions: Parameters<typeof getBatchInstruction>[0],
        config?: Parameters<typeof getBatchInstruction>[1],
    ) => ReturnType<typeof getBatchInstruction> & SelfPlanAndSendFunctions;
    /** Create a new token mint. */
    createMint: (
        input: MakeOptional<CreateMintInstructionPlanInput, 'payer'>,
        config?: CreateMintInstructionPlanConfig,
    ) => ReturnType<typeof getCreateMintInstructionPlan> & SelfPlanAndSendFunctions;
    /** Mint tokens to an owner's ATA (created if needed). */
    mintToATA: (
        input: MakeOptional<MintToATAInstructionPlanAsyncInput, 'payer'>,
        config?: MintToATAInstructionPlanConfig,
    ) => ReturnType<typeof getMintToATAInstructionPlanAsync> & SelfPlanAndSendFunctions;
    /** Transfer tokens to a recipient's ATA (created if needed). */
    transferToATA: (
        input: MakeOptional<TransferToATAInstructionPlanAsyncInput, 'payer'>,
        config?: TransferToATAInstructionPlanConfig,
    ) => ReturnType<typeof getTransferToATAInstructionPlanAsync> & SelfPlanAndSendFunctions;
};

export function tokenProgram() {
    return <T extends TokenPluginRequirements>(client: T) => {
        return pipe(client, generatedTokenProgram(), c => ({
            ...c,
            token: <TokenPlugin>{
                ...c.token,
                instructions: {
                    ...c.token.instructions,
                    batch: (input, config) => addSelfPlanAndSendFunctions(client, getBatchInstruction(input, config)),
                    createMint: (input, config) =>
                        addSelfPlanAndSendFunctions(
                            client,
                            getCreateMintInstructionPlan({ ...input, payer: input.payer ?? client.payer }, config),
                        ),
                    mintToATA: (input, config) =>
                        addSelfPlanAndSendFunctions(
                            client,
                            getMintToATAInstructionPlanAsync({ ...input, payer: input.payer ?? client.payer }, config),
                        ),
                    transferToATA: (input, config) =>
                        addSelfPlanAndSendFunctions(
                            client,
                            getTransferToATAInstructionPlanAsync(
                                { ...input, payer: input.payer ?? client.payer },
                                config,
                            ),
                        ),
                },
            },
        }));
    };
}
