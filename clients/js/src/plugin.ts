import { ClientWithPayer, pipe } from '@solana/kit';
import { addSelfPlanAndSendFunctions, SelfPlanAndSendFunctions } from '@solana/kit/program-client-core';

import { CreateMintInstructionPlanInput, getCreateMintInstructionPlan } from './createMint';
import {
    TokenPlugin as GeneratedTokenPlugin,
    TokenPluginInstructions as GeneratedTokenPluginInstructions,
    TokenPluginRequirements as GeneratedTokenPluginRequirements,
    tokenProgram as generatedTokenProgram,
} from './generated';
import { getMintToATAInstructionPlan, MintToATAInstructionPlanInput } from './mintToATA';
import { getTransferToATAInstructionPlan, TransferToATAInstructionPlanInput } from './transferToATA';

export type TokenPluginRequirements = GeneratedTokenPluginRequirements & ClientWithPayer;

export type TokenPlugin = Omit<GeneratedTokenPlugin, 'instructions'> & { instructions: TokenPluginInstructions };

export type TokenPluginInstructions = GeneratedTokenPluginInstructions & {
    createMint: (
        input: MakeOptional<CreateMintInstructionPlanInput, 'payer'>,
    ) => ReturnType<typeof getCreateMintInstructionPlan> & SelfPlanAndSendFunctions;
    mintToATA: (
        input: MakeOptional<MintToATAInstructionPlanInput, 'payer'>,
    ) => ReturnType<typeof getMintToATAInstructionPlan> & SelfPlanAndSendFunctions;
    transferToATA: (
        input: MakeOptional<TransferToATAInstructionPlanInput, 'payer'>,
    ) => ReturnType<typeof getTransferToATAInstructionPlan> & SelfPlanAndSendFunctions;
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
                            getCreateMintInstructionPlan({ ...input, payer: input.payer ?? client.payer }),
                        ),
                    mintToATA: input =>
                        addSelfPlanAndSendFunctions(
                            client,
                            getMintToATAInstructionPlan({ ...input, payer: input.payer ?? client.payer }),
                        ),
                    transferToATA: input =>
                        addSelfPlanAndSendFunctions(
                            client,
                            getTransferToATAInstructionPlan({ ...input, payer: input.payer ?? client.payer }),
                        ),
                },
            },
        }));
    };
}

type MakeOptional<T, K extends keyof T> = Omit<T, K> & Partial<Pick<T, K>>;
