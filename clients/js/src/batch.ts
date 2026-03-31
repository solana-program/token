import { AccountMeta, Address, Instruction, InstructionWithData, ReadonlyUint8Array } from '@solana/kit';
import {
    BatchInstruction,
    getBatchInstruction as innerGetBatchInstruction,
    parseBatchInstruction as innerParseBatchInstruction,
    ParsedBatchInstruction,
    ParsedTokenInstruction,
    parseTokenInstruction,
    TOKEN_PROGRAM_ADDRESS,
} from './generated';

export function getBatchInstruction<TProgramAddress extends Address = typeof TOKEN_PROGRAM_ADDRESS>(
    instructions: Instruction<TProgramAddress>[],
    config?: { programAddress?: TProgramAddress },
): BatchInstruction<TProgramAddress, AccountMeta<string>[]> {
    const accounts = instructions.flatMap(instruction => instruction.accounts ?? []);
    const data = instructions.map(instruction => ({
        numberOfAccounts: instruction.accounts?.length ?? 0,
        instructionData: instruction.data ?? new Uint8Array(),
    }));

    return Object.freeze({ ...innerGetBatchInstruction<TProgramAddress>({ data }, config), accounts });
}

export function parseBatchInstruction<TProgram extends string>(
    instruction: Instruction<TProgram> & InstructionWithData<ReadonlyUint8Array>,
): ParsedBatchInstruction<TProgram> & { instructions: ParsedTokenInstruction<TProgram>[] } {
    const rawBatchInstruction = innerParseBatchInstruction(instruction);
    let accountOffset = 0;
    const instructions = rawBatchInstruction.data.data.map(({ numberOfAccounts, instructionData }) => {
        const innerInstruction = parseTokenInstruction<TProgram>({
            programAddress: instruction.programAddress,
            data: instructionData,
            accounts: instruction.accounts?.slice(accountOffset, accountOffset + numberOfAccounts) ?? [],
        });
        accountOffset += numberOfAccounts;
        return innerInstruction;
    });

    return { ...rawBatchInstruction, instructions };
}
