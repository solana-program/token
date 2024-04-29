/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/metaplex-foundation/kinobi
 */

import { Address } from '@solana/web3.js';
import {
  ParsedCreateAssociatedTokenIdempotentInstruction,
  ParsedCreateAssociatedTokenInstruction,
  ParsedRecoverNestedAssociatedTokenInstruction,
} from '../instructions';

export const ASSOCIATED_TOKEN_PROGRAM_ADDRESS =
  'ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL' as Address<'ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL'>;

export enum AssociatedTokenInstruction {
  CreateAssociatedToken,
  CreateAssociatedTokenIdempotent,
  RecoverNestedAssociatedToken,
}

export type ParsedAssociatedTokenInstruction<
  TProgram extends string = 'ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL',
> =
  | ({
      instructionType: AssociatedTokenInstruction.CreateAssociatedToken;
    } & ParsedCreateAssociatedTokenInstruction<TProgram>)
  | ({
      instructionType: AssociatedTokenInstruction.CreateAssociatedTokenIdempotent;
    } & ParsedCreateAssociatedTokenIdempotentInstruction<TProgram>)
  | ({
      instructionType: AssociatedTokenInstruction.RecoverNestedAssociatedToken;
    } & ParsedRecoverNestedAssociatedTokenInstruction<TProgram>);
