import { ClientWithPayer, pipe } from '@solana/kit';
import {
    tokenProgram as generatedTokenProgram,
    TokenPlugin as GeneratedTokenPlugin,
    TokenPluginInstructions as GeneratedTokenPluginInstructions,
    TokenPluginRequirements as GeneratedTokenPluginRequirements,
} from './generated';
import { CreateMintInstructionPlanInput, getCreateMintInstructionPlan } from './createMint';
import { addSelfPlanAndSendFunctions, SelfPlanAndSendFunctions } from '@solana/kit/program-client-core';

export type TokenPluginRequirements = GeneratedTokenPluginRequirements & ClientWithPayer;

export type TokenPlugin = Omit<GeneratedTokenPlugin, 'instructions'> & { instructions: TokenPluginInstructions };

export type TokenPluginInstructions = GeneratedTokenPluginInstructions & {
    createMint: (
        input: MakeOptional<CreateMintInstructionPlanInput, 'payer'>,
    ) => ReturnType<typeof getCreateMintInstructionPlan> & SelfPlanAndSendFunctions;
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
                },
            },
        }));
    };
}

type MakeOptional<T, K extends keyof T> = Omit<T, K> & Partial<Pick<T, K>>;
