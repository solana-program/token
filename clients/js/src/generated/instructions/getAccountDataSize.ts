/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/kinobi-so/kinobi
 */

import {
  combineCodec,
  getStructDecoder,
  getStructEncoder,
  getU8Decoder,
  getU8Encoder,
  transformEncoder,
  type Address,
  type Codec,
  type Decoder,
  type Encoder,
  type IAccountMeta,
  type IInstruction,
  type IInstructionWithAccounts,
  type IInstructionWithData,
  type ReadonlyAccount,
} from '@solana/web3.js';
import { TOKEN_PROGRAM_ADDRESS } from '../programs';
import { getAccountMetaFactory, type ResolvedAccount } from '../shared';

export const GET_ACCOUNT_DATA_SIZE_DISCRIMINATOR = 21;

export function getGetAccountDataSizeDiscriminatorBytes() {
  return getU8Encoder().encode(GET_ACCOUNT_DATA_SIZE_DISCRIMINATOR);
}

export type GetAccountDataSizeInstruction<
  TProgram extends string = typeof TOKEN_PROGRAM_ADDRESS,
  TAccountMint extends string | IAccountMeta<string> = string,
  TRemainingAccounts extends readonly IAccountMeta<string>[] = [],
> = IInstruction<TProgram> &
  IInstructionWithData<Uint8Array> &
  IInstructionWithAccounts<
    [
      TAccountMint extends string
        ? ReadonlyAccount<TAccountMint>
        : TAccountMint,
      ...TRemainingAccounts,
    ]
  >;

export type GetAccountDataSizeInstructionData = { discriminator: number };

export type GetAccountDataSizeInstructionDataArgs = {};

export function getGetAccountDataSizeInstructionDataEncoder(): Encoder<GetAccountDataSizeInstructionDataArgs> {
  return transformEncoder(
    getStructEncoder([['discriminator', getU8Encoder()]]),
    (value) => ({
      ...value,
      discriminator: GET_ACCOUNT_DATA_SIZE_DISCRIMINATOR,
    })
  );
}

export function getGetAccountDataSizeInstructionDataDecoder(): Decoder<GetAccountDataSizeInstructionData> {
  return getStructDecoder([['discriminator', getU8Decoder()]]);
}

export function getGetAccountDataSizeInstructionDataCodec(): Codec<
  GetAccountDataSizeInstructionDataArgs,
  GetAccountDataSizeInstructionData
> {
  return combineCodec(
    getGetAccountDataSizeInstructionDataEncoder(),
    getGetAccountDataSizeInstructionDataDecoder()
  );
}

export type GetAccountDataSizeInput<TAccountMint extends string = string> = {
  /** The mint to calculate for. */
  mint: Address<TAccountMint>;
};

export function getGetAccountDataSizeInstruction<
  TAccountMint extends string,
  TProgramAddress extends Address = typeof TOKEN_PROGRAM_ADDRESS,
>(
  input: GetAccountDataSizeInput<TAccountMint>,
  config?: { programAddress?: TProgramAddress }
): GetAccountDataSizeInstruction<TProgramAddress, TAccountMint> {
  // Program address.
  const programAddress = config?.programAddress ?? TOKEN_PROGRAM_ADDRESS;

  // Original accounts.
  const originalAccounts = {
    mint: { value: input.mint ?? null, isWritable: false },
  };
  const accounts = originalAccounts as Record<
    keyof typeof originalAccounts,
    ResolvedAccount
  >;

  const getAccountMeta = getAccountMetaFactory(programAddress, 'programId');
  const instruction = {
    accounts: [getAccountMeta(accounts.mint)],
    programAddress,
    data: getGetAccountDataSizeInstructionDataEncoder().encode({}),
  } as GetAccountDataSizeInstruction<TProgramAddress, TAccountMint>;

  return instruction;
}

export type ParsedGetAccountDataSizeInstruction<
  TProgram extends string = typeof TOKEN_PROGRAM_ADDRESS,
  TAccountMetas extends readonly IAccountMeta[] = readonly IAccountMeta[],
> = {
  programAddress: Address<TProgram>;
  accounts: {
    /** The mint to calculate for. */
    mint: TAccountMetas[0];
  };
  data: GetAccountDataSizeInstructionData;
};

export function parseGetAccountDataSizeInstruction<
  TProgram extends string,
  TAccountMetas extends readonly IAccountMeta[],
>(
  instruction: IInstruction<TProgram> &
    IInstructionWithAccounts<TAccountMetas> &
    IInstructionWithData<Uint8Array>
): ParsedGetAccountDataSizeInstruction<TProgram, TAccountMetas> {
  if (instruction.accounts.length < 1) {
    // TODO: Coded error.
    throw new Error('Not enough accounts');
  }
  let accountIndex = 0;
  const getNextAccount = () => {
    const accountMeta = instruction.accounts![accountIndex]!;
    accountIndex += 1;
    return accountMeta;
  };
  return {
    programAddress: instruction.programAddress,
    accounts: {
      mint: getNextAccount(),
    },
    data: getGetAccountDataSizeInstructionDataDecoder().decode(
      instruction.data
    ),
  };
}
