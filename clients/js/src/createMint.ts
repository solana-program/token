import { getCreateAccountInstruction } from '@solana-program/system';
import {
    Address,
    ClientWithGetMinimumBalance,
    InstructionPlan,
    OptionOrNullable,
    sequentialInstructionPlan,
    TransactionSigner,
} from '@solana/kit';

import { getInitializeMint2Instruction, getMintSize, TOKEN_PROGRAM_ADDRESS } from './generated';

export type CreateMintInstructionPlanInput = {
    /** Funding account (must be a system account). */
    payer: TransactionSigner;
    /** New mint account to create. */
    newMint: TransactionSigner;
    /** Number of base 10 digits to the right of the decimal place. */
    decimals: number;
    /** The authority/multisignature to mint tokens. */
    mintAuthority: Address;
    /** The optional freeze authority/multisignature of the mint. */
    freezeAuthority?: OptionOrNullable<Address>;
    /**
     * Optional override for the amount of Lamports to fund the mint account with.
     * @default enough to make the account rent-exempt (currently 1,461,600 Lamports)
     *  */
    mintAccountLamports?: number;
};

export type CreateMintInstructionPlanConfig = {
    systemProgram?: Address;
    tokenProgram?: Address;
};

export async function getCreateMintInstructionPlan(
    client: ClientWithGetMinimumBalance,
    input: CreateMintInstructionPlanInput,
    config?: CreateMintInstructionPlanConfig,
): Promise<InstructionPlan> {
    const space = getMintSize();
    const lamports = input.mintAccountLamports ?? (await client.getMinimumBalance(space));
    return sequentialInstructionPlan([
        getCreateAccountInstruction(
            {
                payer: input.payer,
                newAccount: input.newMint,
                lamports,
                space,
                programAddress: config?.tokenProgram ?? TOKEN_PROGRAM_ADDRESS,
            },
            {
                programAddress: config?.systemProgram,
            },
        ),
        getInitializeMint2Instruction(
            {
                mint: input.newMint.address,
                decimals: input.decimals,
                mintAuthority: input.mintAuthority,
                freezeAuthority: input.freezeAuthority,
            },
            {
                programAddress: config?.tokenProgram,
            },
        ),
    ]);
}
