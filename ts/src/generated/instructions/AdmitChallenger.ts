/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */

import * as web3 from '@solana/web3.js'
import * as beetSolana from '@metaplex-foundation/beet-solana'
import * as beet from '@metaplex-foundation/beet'

/**
 * @category Instructions
 * @category AdmitChallenger
 * @category generated
 */
export type AdmitChallengerInstructionArgs = {
  challengePda: web3.PublicKey
}
/**
 * @category Instructions
 * @category AdmitChallenger
 * @category generated
 */
export const AdmitChallengerStruct = new beet.BeetArgsStruct<
  AdmitChallengerInstructionArgs & {
    instructionDiscriminator: number
  }
>(
  [
    ['instructionDiscriminator', beet.u8],
    ['challengePda', beetSolana.publicKey],
  ],
  'AdmitChallengerInstructionArgs'
)
/**
 * Accounts required by the _AdmitChallenger_ instruction
 *
 * @property [_writable_, **signer**] payer pays for the transaction
 * @property [_writable_] creator challenge authority
 * @property [] challengePda PDA for the challenge
 * @property [] challenger challenger account which receives the redeemed token
 * @property [_writable_] challengerPda PDA for the challenger
 * @category Instructions
 * @category AdmitChallenger
 * @category generated
 */
export type AdmitChallengerInstructionAccounts = {
  payer: web3.PublicKey
  creator: web3.PublicKey
  challengePda: web3.PublicKey
  challenger: web3.PublicKey
  challengerPda: web3.PublicKey
  systemProgram?: web3.PublicKey
}

export const admitChallengerInstructionDiscriminator = 3

/**
 * Creates a _AdmitChallenger_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 *
 * @category Instructions
 * @category AdmitChallenger
 * @category generated
 */
export function createAdmitChallengerInstruction(
  accounts: AdmitChallengerInstructionAccounts,
  args: AdmitChallengerInstructionArgs,
  programId = new web3.PublicKey('FFFFaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS')
) {
  const [data] = AdmitChallengerStruct.serialize({
    instructionDiscriminator: admitChallengerInstructionDiscriminator,
    ...args,
  })
  const keys: web3.AccountMeta[] = [
    {
      pubkey: accounts.payer,
      isWritable: true,
      isSigner: true,
    },
    {
      pubkey: accounts.creator,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.challengePda,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.challenger,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.challengerPda,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.systemProgram ?? web3.SystemProgram.programId,
      isWritable: false,
      isSigner: false,
    },
  ]

  const ix = new web3.TransactionInstruction({
    programId,
    keys,
    data,
  })
  return ix
}
