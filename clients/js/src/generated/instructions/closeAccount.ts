/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/kinobi-so/kinobi
 */

import {
  AccountRole,
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
  type IAccountSignerMeta,
  type IInstruction,
  type IInstructionWithAccounts,
  type IInstructionWithData,
  type ReadonlyAccount,
  type ReadonlySignerAccount,
  type TransactionSigner,
  type WritableAccount,
} from '@solana/web3.js';
import { TOKEN_PROGRAM_ADDRESS } from '../programs';
import { getAccountMetaFactory, type ResolvedAccount } from '../shared';

export const CLOSE_ACCOUNT_DISCRIMINATOR = 9;

export function getCloseAccountDiscriminatorBytes() {
  return getU8Encoder().encode(CLOSE_ACCOUNT_DISCRIMINATOR);
}

export type CloseAccountInstruction<
  TProgram extends string = typeof TOKEN_PROGRAM_ADDRESS,
  TAccountAccount extends string | IAccountMeta<string> = string,
  TAccountDestination extends string | IAccountMeta<string> = string,
  TAccountOwner extends string | IAccountMeta<string> = string,
  TRemainingAccounts extends readonly IAccountMeta<string>[] = [],
> = IInstruction<TProgram> &
  IInstructionWithData<Uint8Array> &
  IInstructionWithAccounts<
    [
      TAccountAccount extends string
        ? WritableAccount<TAccountAccount>
        : TAccountAccount,
      TAccountDestination extends string
        ? WritableAccount<TAccountDestination>
        : TAccountDestination,
      TAccountOwner extends string
        ? ReadonlyAccount<TAccountOwner>
        : TAccountOwner,
      ...TRemainingAccounts,
    ]
  >;

export type CloseAccountInstructionData = { discriminator: number };

export type CloseAccountInstructionDataArgs = {};

export function getCloseAccountInstructionDataEncoder(): Encoder<CloseAccountInstructionDataArgs> {
  return transformEncoder(
    getStructEncoder([['discriminator', getU8Encoder()]]),
    (value) => ({ ...value, discriminator: CLOSE_ACCOUNT_DISCRIMINATOR })
  );
}

export function getCloseAccountInstructionDataDecoder(): Decoder<CloseAccountInstructionData> {
  return getStructDecoder([['discriminator', getU8Decoder()]]);
}

export function getCloseAccountInstructionDataCodec(): Codec<
  CloseAccountInstructionDataArgs,
  CloseAccountInstructionData
> {
  return combineCodec(
    getCloseAccountInstructionDataEncoder(),
    getCloseAccountInstructionDataDecoder()
  );
}

export type CloseAccountInput<
  TAccountAccount extends string = string,
  TAccountDestination extends string = string,
  TAccountOwner extends string = string,
> = {
  /** The account to close. */
  account: Address<TAccountAccount>;
  /** The destination account. */
  destination: Address<TAccountDestination>;
  /** The account's owner or its multisignature account. */
  owner: Address<TAccountOwner> | TransactionSigner<TAccountOwner>;
  multiSigners?: Array<TransactionSigner>;
};

export function getCloseAccountInstruction<
  TAccountAccount extends string,
  TAccountDestination extends string,
  TAccountOwner extends string,
  TProgramAddress extends Address = typeof TOKEN_PROGRAM_ADDRESS,
>(
  input: CloseAccountInput<TAccountAccount, TAccountDestination, TAccountOwner>,
  config?: { programAddress?: TProgramAddress }
): CloseAccountInstruction<
  TProgramAddress,
  TAccountAccount,
  TAccountDestination,
  (typeof input)['owner'] extends TransactionSigner<TAccountOwner>
    ? ReadonlySignerAccount<TAccountOwner> & IAccountSignerMeta<TAccountOwner>
    : TAccountOwner
> {
  // Program address.
  const programAddress = config?.programAddress ?? TOKEN_PROGRAM_ADDRESS;

  // Original accounts.
  const originalAccounts = {
    account: { value: input.account ?? null, isWritable: true },
    destination: { value: input.destination ?? null, isWritable: true },
    owner: { value: input.owner ?? null, isWritable: false },
  };
  const accounts = originalAccounts as Record<
    keyof typeof originalAccounts,
    ResolvedAccount
  >;

  // Original args.
  const args = { ...input };

  // Remaining accounts.
  const remainingAccounts: IAccountMeta[] = (args.multiSigners ?? []).map(
    (signer) => ({
      address: signer.address,
      role: AccountRole.READONLY_SIGNER,
      signer,
    })
  );

  const getAccountMeta = getAccountMetaFactory(programAddress, 'programId');
  const instruction = {
    accounts: [
      getAccountMeta(accounts.account),
      getAccountMeta(accounts.destination),
      getAccountMeta(accounts.owner),
      ...remainingAccounts,
    ],
    programAddress,
    data: getCloseAccountInstructionDataEncoder().encode({}),
  } as CloseAccountInstruction<
    TProgramAddress,
    TAccountAccount,
    TAccountDestination,
    (typeof input)['owner'] extends TransactionSigner<TAccountOwner>
      ? ReadonlySignerAccount<TAccountOwner> & IAccountSignerMeta<TAccountOwner>
      : TAccountOwner
  >;

  return instruction;
}

export type ParsedCloseAccountInstruction<
  TProgram extends string = typeof TOKEN_PROGRAM_ADDRESS,
  TAccountMetas extends readonly IAccountMeta[] = readonly IAccountMeta[],
> = {
  programAddress: Address<TProgram>;
  accounts: {
    /** The account to close. */
    account: TAccountMetas[0];
    /** The destination account. */
    destination: TAccountMetas[1];
    /** The account's owner or its multisignature account. */
    owner: TAccountMetas[2];
  };
  data: CloseAccountInstructionData;
};

export function parseCloseAccountInstruction<
  TProgram extends string,
  TAccountMetas extends readonly IAccountMeta[],
>(
  instruction: IInstruction<TProgram> &
    IInstructionWithAccounts<TAccountMetas> &
    IInstructionWithData<Uint8Array>
): ParsedCloseAccountInstruction<TProgram, TAccountMetas> {
  if (instruction.accounts.length < 3) {
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
      account: getNextAccount(),
      destination: getNextAccount(),
      owner: getNextAccount(),
    },
    data: getCloseAccountInstructionDataDecoder().decode(instruction.data),
  };
}
