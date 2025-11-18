import { getCreateAccountInstruction } from '@solana-program/system';
import {
  Address,
  InstructionPlan,
  OptionOrNullable,
  sequentialInstructionPlan,
  TransactionSigner,
} from '@solana/kit';
import {
  getInitializeMint2Instruction,
  getMintSize,
  TOKEN_PROGRAM_ADDRESS,
} from './generated';

// RPC `getMinimumBalanceForRentExemption` for 82 bytes, which is token mint size
// Hardcoded to avoid requiring an RPC request each time
const MINIMUM_BALANCE_FOR_MINT = 1461600;

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
   * @default 1461600
   *  */
  mintAccountLamports?: number;
};

type CreateMintInstructionPlanConfig = {
  systemProgramAddress?: Address;
  tokenProgramAddress?: Address;
};

export function createMintInstructionPlan(
  params: CreateMintInstructionPlanInput,
  config?: CreateMintInstructionPlanConfig
): InstructionPlan {
  return sequentialInstructionPlan([
    getCreateAccountInstruction(
      {
        payer: params.payer,
        newAccount: params.newMint,
        lamports: params.mintAccountLamports ?? MINIMUM_BALANCE_FOR_MINT,
        space: getMintSize(),
        programAddress: config?.tokenProgramAddress ?? TOKEN_PROGRAM_ADDRESS,
      },
      {
        programAddress: config?.systemProgramAddress,
      }
    ),
    getInitializeMint2Instruction(
      {
        mint: params.newMint.address,
        decimals: params.decimals,
        mintAuthority: params.mintAuthority,
        freezeAuthority: params.freezeAuthority,
      },
      {
        programAddress: config?.tokenProgramAddress,
      }
    ),
  ]);
}
