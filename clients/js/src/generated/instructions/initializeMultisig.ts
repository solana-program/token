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
  type WritableAccount,
} from '@solana/kit';
import { TOKEN_PROGRAM_ADDRESS } from '../programs';
import { getAccountMetaFactory, type ResolvedAccount } from '../shared';

export const INITIALIZE_MULTISIG_DISCRIMINATOR = 2;

export function getInitializeMultisigDiscriminatorBytes() {
  return getU8Encoder().encode(INITIALIZE_MULTISIG_DISCRIMINATOR);
}

export type InitializeMultisigInstruction<
  TProgram extends string = typeof TOKEN_PROGRAM_ADDRESS,
  TAccountMultisig extends string | IAccountMeta<string> = string,
  TAccountRent extends
    | string
    | IAccountMeta<string> = 'SysvarRent111111111111111111111111111111111',
  TRemainingAccounts extends readonly IAccountMeta<string>[] = [],
> = IInstruction<TProgram> &
  IInstructionWithData<Uint8Array> &
  IInstructionWithAccounts<
    [
      TAccountMultisig extends string
        ? WritableAccount<TAccountMultisig>
        : TAccountMultisig,
      TAccountRent extends string
        ? ReadonlyAccount<TAccountRent>
        : TAccountRent,
      ...TRemainingAccounts,
    ]
  >;

export type InitializeMultisigInstructionData = {
  discriminator: number;
  /** The number of signers (M) required to validate this multisignature account. */
  m: number;
};

export type InitializeMultisigInstructionDataArgs = {
  /** The number of signers (M) required to validate this multisignature account. */
  m: number;
};

export function getInitializeMultisigInstructionDataEncoder(): Encoder<InitializeMultisigInstructionDataArgs> {
  return transformEncoder(
    getStructEncoder([
      ['discriminator', getU8Encoder()],
      ['m', getU8Encoder()],
    ]),
    (value) => ({ ...value, discriminator: INITIALIZE_MULTISIG_DISCRIMINATOR })
  );
}

export function getInitializeMultisigInstructionDataDecoder(): Decoder<InitializeMultisigInstructionData> {
  return getStructDecoder([
    ['discriminator', getU8Decoder()],
    ['m', getU8Decoder()],
  ]);
}

export function getInitializeMultisigInstructionDataCodec(): Codec<
  InitializeMultisigInstructionDataArgs,
  InitializeMultisigInstructionData
> {
  return combineCodec(
    getInitializeMultisigInstructionDataEncoder(),
    getInitializeMultisigInstructionDataDecoder()
  );
}

export type InitializeMultisigInput<
  TAccountMultisig extends string = string,
  TAccountRent extends string = string,
> = {
  /** The multisignature account to initialize. */
  multisig: Address<TAccountMultisig>;
  /** Rent sysvar. */
  rent?: Address<TAccountRent>;
  m: InitializeMultisigInstructionDataArgs['m'];
  signers: Array<Address>;
};

export function getInitializeMultisigInstruction<
  TAccountMultisig extends string,
  TAccountRent extends string,
  TProgramAddress extends Address = typeof TOKEN_PROGRAM_ADDRESS,
>(
  input: InitializeMultisigInput<TAccountMultisig, TAccountRent>,
  config?: { programAddress?: TProgramAddress }
): InitializeMultisigInstruction<
  TProgramAddress,
  TAccountMultisig,
  TAccountRent
> {
  // Program address.
  const programAddress = config?.programAddress ?? TOKEN_PROGRAM_ADDRESS;

  // Original accounts.
  const originalAccounts = {
    multisig: { value: input.multisig ?? null, isWritable: true },
    rent: { value: input.rent ?? null, isWritable: false },
  };
  const accounts = originalAccounts as Record<
    keyof typeof originalAccounts,
    ResolvedAccount
  >;

  // Original args.
  const args = { ...input };

  // Resolve default values.
  if (!accounts.rent.value) {
    accounts.rent.value =
      'SysvarRent111111111111111111111111111111111' as Address<'SysvarRent111111111111111111111111111111111'>;
  }

  // Remaining accounts.
  const remainingAccounts: IAccountMeta[] = args.signers.map((address) => ({
    address,
    role: AccountRole.READONLY,
  }));

  const getAccountMeta = getAccountMetaFactory(programAddress, 'programId');
  const instruction = {
    accounts: [
      getAccountMeta(accounts.multisig),
      getAccountMeta(accounts.rent),
      ...remainingAccounts,
    ],
    programAddress,
    data: getInitializeMultisigInstructionDataEncoder().encode(
      args as InitializeMultisigInstructionDataArgs
    ),
  } as InitializeMultisigInstruction<
    TProgramAddress,
    TAccountMultisig,
    TAccountRent
  >;

  return instruction;
}

export type ParsedInitializeMultisigInstruction<
  TProgram extends string = typeof TOKEN_PROGRAM_ADDRESS,
  TAccountMetas extends readonly IAccountMeta[] = readonly IAccountMeta[],
> = {
  programAddress: Address<TProgram>;
  accounts: {
    /** The multisignature account to initialize. */
    multisig: TAccountMetas[0];
    /** Rent sysvar. */
    rent: TAccountMetas[1];
  };
  data: InitializeMultisigInstructionData;
};

export function parseInitializeMultisigInstruction<
  TProgram extends string,
  TAccountMetas extends readonly IAccountMeta[],
>(
  instruction: IInstruction<TProgram> &
    IInstructionWithAccounts<TAccountMetas> &
    IInstructionWithData<Uint8Array>
): ParsedInitializeMultisigInstruction<TProgram, TAccountMetas> {
  if (instruction.accounts.length < 2) {
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
      multisig: getNextAccount(),
      rent: getNextAccount(),
    },
    data: getInitializeMultisigInstructionDataDecoder().decode(
      instruction.data
    ),
  };
}
