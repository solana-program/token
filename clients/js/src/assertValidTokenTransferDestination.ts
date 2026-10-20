import { SYSTEM_PROGRAM_ADDRESS } from '@solana-program/system';
import {
    fetchEncodedAccount,
    isOffCurveAddress,
    type Address,
    type FetchAccountConfig,
    type GetAccountInfoApi,
    type Rpc,
} from '@solana/kit';
import {
    getAddressFromResolvedInstructionAccount,
    type InstructionAccountInput,
} from '@solana/kit/program-client-core';

import { AccountState, decodeToken, findAssociatedTokenPda, getTokenSize, TOKEN_PROGRAM_ADDRESS } from './generated';

/** Distinguishes why a destination was rejected by {@link assertValidTokenTransferDestination}. */
export type InvalidTokenTransferDestinationReason =
    | 'unexpected-owner'
    | 'unfunded-recipient'
    | 'off-curve'
    | 'ata-mismatch'
    | 'missing-token-account'
    | 'not-token-account'
    | 'mint-mismatch'
    | 'token-owner-mismatch';

/** Details attached to an {@link InvalidTokenTransferDestinationError}, depending on its `reason`. */
export type InvalidTokenTransferDestinationDetails = {
    /** The program that owns the recipient account. Set when `reason` is `'unexpected-owner'`. */
    owner?: Address;
    /** The mint the destination token account holds. Set when `reason` is `'mint-mismatch'`. */
    mint?: Address;
    /** The current owner of the destination token account. Set when `reason` is `'token-owner-mismatch'`. */
    tokenOwner?: Address;
};

/** Thrown by {@link assertValidTokenTransferDestination} when a destination is rejected. */
export class InvalidTokenTransferDestinationError extends Error {
    override readonly name = 'InvalidTokenTransferDestinationError';
    readonly owner?: Address;
    readonly mint?: Address;
    readonly tokenOwner?: Address;

    constructor(
        message: string,
        readonly destination: Address,
        readonly recipient: Address,
        readonly reason: InvalidTokenTransferDestinationReason,
        details: InvalidTokenTransferDestinationDetails = {},
    ) {
        super(message);
        this.owner = details.owner;
        this.mint = details.mint;
        this.tokenOwner = details.tokenOwner;
    }
}

export type AssertValidTokenTransferDestinationInput = {
    /** The token mint to transfer. */
    mint: InstructionAccountInput;
    /**
     * Associated token account address to transfer to, which may not exist yet. With
     * `allowNonAssociatedTokenAccount`, any existing token account owned by `recipient`.
     * Note: Use {@link findAssociatedTokenPda} to derive the associated token account address.
     */
    destination: InstructionAccountInput;
    /** Wallet address for the destination. */
    recipient: InstructionAccountInput;
};

/**
 * Options for {@link assertValidTokenTransferDestination}. Any {@link FetchAccountConfig} fields are passed
 * through to the account fetches.
 */
export type AssertValidTokenTransferDestinationConfig = FetchAccountConfig & {
    /**
     * Allow a recipient that has no account on-chain yet. Defaults to `false`, since an unfunded recipient
     * is often a mistyped address.
     */
    allowUnfundedRecipient?: boolean;
    /**
     * Allow an off-curve (program-derived) recipient, existing or not. Defaults to `true`, since
     * program-controlled wallets such as multisigs are common, legitimate recipients; set to `false` to
     * reject off-curve recipients until they confirm they control them.
     */
    allowOffCurve?: boolean;
    /**
     * The program an existing recipient account must be owned by. Defaults to the System Program
     * ({@link SYSTEM_PROGRAM_ADDRESS}).
     */
    programOwner?: Address;
    /**
     * Allow a destination that is not the recipient's associated token account, as long as it is an
     * existing token account for `mint` owned by `recipient`. Defaults to `false`. Such a destination cannot
     * be used with `getTransferToATAInstructionPlan`; transfer to it with `getTransferCheckedInstruction`.
     */
    allowNonAssociatedTokenAccount?: boolean;
    /**
     * Allow an associated token account destination that does not exist yet, creating it as part of the
     * transfer. Defaults to `true`, since a recipient's first transfer of a mint is the common case; set to
     * `false` to avoid paying its rent or to require that the recipient has held the mint before.
     */
    allowMissingAssociatedTokenAccount?: boolean;
};

/**
 * Asserts that `recipient` can safely receive a transfer of `mint` at `destination`, its associated token
 * account, by fetching both accounts. Matches the inputs of `getTransferToATAInstructionPlan`. Otherwise
 * throws an {@link InvalidTokenTransferDestinationError} with one of these reasons:
 *
 * - `'unexpected-owner'`: the recipient account exists but is not owned by `programOwner` (the System
 *   Program by default), e.g. a token account or mint passed as the recipient.
 * - `'unfunded-recipient'`: the recipient has no account on-chain, which often means a mistyped address.
 *   Skipped when `allowUnfundedRecipient` is set.
 * - `'off-curve'`: the recipient is off-curve, existing or not, and may be a smart wallet they must confirm
 *   they control. Only checked when `allowOffCurve` is `false`.
 * - `'ata-mismatch'`: `destination` is not the recipient's associated token account for `mint`. Skipped
 *   when `allowNonAssociatedTokenAccount` is set.
 * - `'missing-token-account'`: `destination` has no account on-chain. Skipped for an associated token
 *   account unless `allowMissingAssociatedTokenAccount` is `false`.
 * - `'not-token-account'`: a non-associated `destination` is not an initialized token account.
 * - `'mint-mismatch'`: a non-associated `destination` holds a different mint.
 * - `'token-owner-mismatch'`: the destination exists but is not owned by `recipient`, e.g. an associated
 *   token account whose owner was reassigned.
 *
 * Checks run in the order listed, so a destination that fails several reports the first. Since an
 * unfunded recipient cannot have an associated token account, setting `allowMissingAssociatedTokenAccount`
 * to `false` also rejects unfunded recipients.
 */
export async function assertValidTokenTransferDestination(
    rpc: Rpc<GetAccountInfoApi>,
    input: AssertValidTokenTransferDestinationInput,
    config?: AssertValidTokenTransferDestinationConfig,
): Promise<void> {
    const mint = getAddressFromResolvedInstructionAccount('mint', input.mint);
    const destination = getAddressFromResolvedInstructionAccount('destination', input.destination);
    const recipient = getAddressFromResolvedInstructionAccount('recipient', input.recipient);
    const {
        allowOffCurve = true,
        allowUnfundedRecipient = false,
        programOwner = SYSTEM_PROGRAM_ADDRESS,
        allowNonAssociatedTokenAccount = false,
        allowMissingAssociatedTokenAccount = true,
        ...fetchConfig
    } = config ?? {};
    const [recipientAccount, destinationAccount, [ata]] = await Promise.all([
        fetchEncodedAccount(rpc, recipient, fetchConfig),
        fetchEncodedAccount(rpc, destination, fetchConfig),
        findAssociatedTokenPda({ owner: recipient, mint, tokenProgram: TOKEN_PROGRAM_ADDRESS }),
    ]);

    if (recipientAccount.exists) {
        if (recipientAccount.programAddress !== programOwner) {
            throw new InvalidTokenTransferDestinationError(
                `Refusing to transfer tokens to ${recipient}: it is owned by program ${recipientAccount.programAddress}, not the expected program (${programOwner}).`,
                destination,
                recipient,
                'unexpected-owner',
                { owner: recipientAccount.programAddress },
            );
        }
    } else if (!allowUnfundedRecipient) {
        throw new InvalidTokenTransferDestinationError(
            `Refusing to transfer tokens to ${recipient}: it has no account on-chain yet, which often means the address is mistyped. If you intend to send to a new wallet, pass { allowUnfundedRecipient: true }.`,
            destination,
            recipient,
            'unfunded-recipient',
        );
    }

    if (isOffCurveAddress(recipient) && !allowOffCurve) {
        throw new InvalidTokenTransferDestinationError(
            `Refusing to transfer tokens to ${recipient}: it is an off-curve (program-derived) address, which no keypair controls. It may be a program-controlled account such as a multisig or smart wallet; once the recipient confirms they control it, pass { allowOffCurve: true }.`,
            destination,
            recipient,
            'off-curve',
        );
    }

    const isAssociatedTokenAccount = destination === ata;
    if (!isAssociatedTokenAccount && !allowNonAssociatedTokenAccount) {
        throw new InvalidTokenTransferDestinationError(
            `Refusing to transfer tokens to ${destination}: it is not the associated token account of ${recipient} for mint ${mint} (${ata}). To send to another token account owned by the recipient, pass { allowNonAssociatedTokenAccount: true }.`,
            destination,
            recipient,
            'ata-mismatch',
        );
    }

    if (!destinationAccount.exists) {
        if (isAssociatedTokenAccount && allowMissingAssociatedTokenAccount) return;
        throw new InvalidTokenTransferDestinationError(
            isAssociatedTokenAccount
                ? `Refusing to transfer tokens to ${destination}: the associated token account has no account on-chain yet. To create it as part of the transfer, pass { allowMissingAssociatedTokenAccount: true }.`
                : `Refusing to transfer tokens to ${destination}: it has no account on-chain, and only an associated token account can be created as part of the transfer.`,
            destination,
            recipient,
            'missing-token-account',
        );
    }

    const token =
        destinationAccount.programAddress === TOKEN_PROGRAM_ADDRESS && destinationAccount.data.length === getTokenSize()
            ? decodeToken(destinationAccount).data
            : null;
    if (!token || token.state === AccountState.Uninitialized) {
        throw new InvalidTokenTransferDestinationError(
            `Refusing to transfer tokens to ${destination}: it is not an initialized token account.`,
            destination,
            recipient,
            'not-token-account',
        );
    }
    if (token.mint !== mint) {
        throw new InvalidTokenTransferDestinationError(
            `Refusing to transfer tokens to ${destination}: it holds mint ${token.mint}, not the expected mint (${mint}).`,
            destination,
            recipient,
            'mint-mismatch',
            { mint: token.mint },
        );
    }
    if (token.owner !== recipient) {
        throw new InvalidTokenTransferDestinationError(
            `Refusing to transfer tokens to ${destination}: it is owned by ${token.owner}, not the recipient ${recipient}.`,
            destination,
            recipient,
            'token-owner-mismatch',
            { tokenOwner: token.owner },
        );
    }
}
