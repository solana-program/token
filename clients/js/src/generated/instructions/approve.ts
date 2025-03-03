/**
 * This code was AUTOGENERATED using the codama library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun codama to update it.
 *
 * @see https://github.com/codama-idl/codama
 */

import {
  AccountRole,
  combineCodec,
  getStructDecoder,
  getStructEncoder,
  getU64Decoder,
  getU64Encoder,
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
} from '@solana/kit';
import { TOKEN_PROGRAM_ADDRESS } from '../programs';
import { getAccountMetaFactory, type ResolvedAccount } from '../shared';

export const APPROVE_DISCRIMINATOR = 4;

export function getApproveDiscriminatorBytes() {
  return getU8Encoder().encode(APPROVE_DISCRIMINATOR);
}

export type ApproveInstruction<
  TProgram extends string = typeof TOKEN_PROGRAM_ADDRESS,
  TAccountSource extends string | IAccountMeta<string> = string,
  TAccountDelegate extends string | IAccountMeta<string> = string,
  TAccountOwner extends string | IAccountMeta<string> = string,
  TRemainingAccounts extends readonly IAccountMeta<string>[] = [],
> = IInstruction<TProgram> &
  IInstructionWithData<Uint8Array> &
  IInstructionWithAccounts<
    [
      TAccountSource extends string
        ? WritableAccount<TAccountSource>
        : TAccountSource,
      TAccountDelegate extends string
        ? ReadonlyAccount<TAccountDelegate>
        : TAccountDelegate,
      TAccountOwner extends string
        ? ReadonlyAccount<TAccountOwner>
        : TAccountOwner,
      ...TRemainingAccounts,
    ]
  >;

export type ApproveInstructionData = {
  discriminator: number;
  /** The amount of tokens the delegate is approved for. */
  amount: bigint;
};

export type ApproveInstructionDataArgs = {
  /** The amount of tokens the delegate is approved for. */
  amount: number | bigint;
};

export function getApproveInstructionDataEncoder(): Encoder<ApproveInstructionDataArgs> {
  return transformEncoder(
    getStructEncoder([
      ['discriminator', getU8Encoder()],
      ['amount', getU64Encoder()],
    ]),
    (value) => ({ ...value, discriminator: APPROVE_DISCRIMINATOR })
  );
}

export function getApproveInstructionDataDecoder(): Decoder<ApproveInstructionData> {
  return getStructDecoder([
    ['discriminator', getU8Decoder()],
    ['amount', getU64Decoder()],
  ]);
}

export function getApproveInstructionDataCodec(): Codec<
  ApproveInstructionDataArgs,
  ApproveInstructionData
> {
  return combineCodec(
    getApproveInstructionDataEncoder(),
    getApproveInstructionDataDecoder()
  );
}

export type ApproveInput<
  TAccountSource extends string = string,
  TAccountDelegate extends string = string,
  TAccountOwner extends string = string,
> = {
  /** The source account. */
  source: Address<TAccountSource>;
  /** The delegate. */
  delegate: Address<TAccountDelegate>;
  /** The source account owner or its multisignature account. */
  owner: Address<TAccountOwner> | TransactionSigner<TAccountOwner>;
  amount: ApproveInstructionDataArgs['amount'];
  multiSigners?: Array<TransactionSigner>;
};

export function getApproveInstruction<
  TAccountSource extends string,
  TAccountDelegate extends string,
  TAccountOwner extends string,
  TProgramAddress extends Address = typeof TOKEN_PROGRAM_ADDRESS,
>(
  input: ApproveInput<TAccountSource, TAccountDelegate, TAccountOwner>,
  config?: { programAddress?: TProgramAddress }
): ApproveInstruction<
  TProgramAddress,
  TAccountSource,
  TAccountDelegate,
  (typeof input)['owner'] extends TransactionSigner<TAccountOwner>
    ? ReadonlySignerAccount<TAccountOwner> & IAccountSignerMeta<TAccountOwner>
    : TAccountOwner
> {
  // Program address.
  const programAddress = config?.programAddress ?? TOKEN_PROGRAM_ADDRESS;

  // Original accounts.
  const originalAccounts = {
    source: { value: input.source ?? null, isWritable: true },
    delegate: { value: input.delegate ?? null, isWritable: false },
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
      getAccountMeta(accounts.source),
      getAccountMeta(accounts.delegate),
      getAccountMeta(accounts.owner),
      ...remainingAccounts,
    ],
    programAddress,
    data: getApproveInstructionDataEncoder().encode(
      args as ApproveInstructionDataArgs
    ),
  } as ApproveInstruction<
    TProgramAddress,
    TAccountSource,
    TAccountDelegate,
    (typeof input)['owner'] extends TransactionSigner<TAccountOwner>
      ? ReadonlySignerAccount<TAccountOwner> & IAccountSignerMeta<TAccountOwner>
      : TAccountOwner
  >;

  return instruction;
}

export type ParsedApproveInstruction<
  TProgram extends string = typeof TOKEN_PROGRAM_ADDRESS,
  TAccountMetas extends readonly IAccountMeta[] = readonly IAccountMeta[],
> = {
  programAddress: Address<TProgram>;
  accounts: {
    /** The source account. */
    source: TAccountMetas[0];
    /** The delegate. */
    delegate: TAccountMetas[1];
    /** The source account owner or its multisignature account. */
    owner: TAccountMetas[2];
  };
  data: ApproveInstructionData;
};

export function parseApproveInstruction<
  TProgram extends string,
  TAccountMetas extends readonly IAccountMeta[],
>(
  instruction: IInstruction<TProgram> &
    IInstructionWithAccounts<TAccountMetas> &
    IInstructionWithData<Uint8Array>
): ParsedApproveInstruction<TProgram, TAccountMetas> {
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
      source: getNextAccount(),
      delegate: getNextAccount(),
      owner: getNextAccount(),
    },
    data: getApproveInstructionDataDecoder().decode(instruction.data),
  };
}
