import { InstructionPlan, sequentialInstructionPlan, Address, TransactionSigner } from '@solana/kit';
import {
    findAssociatedTokenPda,
    getCreateAssociatedTokenIdempotentInstruction,
    getMintToCheckedInstruction,
    TOKEN_PROGRAM_ADDRESS,
} from './generated';

export type MintToATAInstructionPlanInput = {
    /** Funding account (must be a system account). */
    payer: TransactionSigner;
    /** Associated token account address to mint to.
     * Will be created if it does not already exist.
     * Note: Use {@link getMintToATAInstructionPlanAsync} instead to derive this automatically.
     * Note: Use {@link findAssociatedTokenPda} to derive the associated token account address.
     */
    ata: Address;
    /** Wallet address for the associated token account. */
    owner: Address;
    /** The token mint for the associated token account. */
    mint: Address;
    /** The mint's minting authority or its multisignature account. */
    mintAuthority: Address | TransactionSigner;
    /** The amount of new tokens to mint. */
    amount: number | bigint;
    /** Expected number of base 10 digits to the right of the decimal place. */
    decimals: number;
    multiSigners?: Array<TransactionSigner>;
};

type MintToATAInstructionPlanConfig = {
    systemProgram?: Address;
    tokenProgram?: Address;
    associatedTokenProgram?: Address;
};

export function getMintToATAInstructionPlan(
    input: MintToATAInstructionPlanInput,
    config?: MintToATAInstructionPlanConfig,
): InstructionPlan {
    return sequentialInstructionPlan([
        getCreateAssociatedTokenIdempotentInstruction(
            {
                payer: input.payer,
                ata: input.ata,
                owner: input.owner,
                mint: input.mint,
                systemProgram: config?.systemProgram,
                tokenProgram: config?.tokenProgram,
            },
            {
                programAddress: config?.associatedTokenProgram,
            },
        ),
        // mint to this token account
        getMintToCheckedInstruction(
            {
                mint: input.mint,
                token: input.ata,
                mintAuthority: input.mintAuthority,
                amount: input.amount,
                decimals: input.decimals,
                multiSigners: input.multiSigners,
            },
            {
                programAddress: config?.tokenProgram,
            },
        ),
    ]);
}

type MintToATAInstructionPlanAsyncInput = Omit<MintToATAInstructionPlanInput, 'ata'>;

export async function getMintToATAInstructionPlanAsync(
    input: MintToATAInstructionPlanAsyncInput,
    config?: MintToATAInstructionPlanConfig,
): Promise<InstructionPlan> {
    const [ataAddress] = await findAssociatedTokenPda({
        owner: input.owner,
        tokenProgram: config?.tokenProgram ?? TOKEN_PROGRAM_ADDRESS,
        mint: input.mint,
    });
    return getMintToATAInstructionPlan(
        {
            ...input,
            ata: ataAddress,
        },
        config,
    );
}
