import { ASSOCIATED_TOKEN_PROGRAM_ID } from '@solana/spl-token'
import { PublicKey } from '@solana/web3.js'
import { pdaForChallenge, pdaForChallenger, pdaForRedeem } from './common/pda'
import { hashSolution, doubleHashSolutions } from './common/solution'
import {
  AddSolutionsInstructionArgs,
  AdmitChallengerInstructionAccounts,
  createAddSolutionsInstruction,
  createAdmitChallengerInstruction,
  CreateChallengeInstructionArgs,
  createCreateChallengeInstruction,
  createRedeemInstruction,
  createStartChallengeInstruction,
  RedeemInstructionAccounts,
} from './generated'
import { Redeem } from './state/redeem'

// -----------------
// Create Challenge
// -----------------
export function createChallenge(
  payer: PublicKey,
  creator: PublicKey,
  id: string,
  admitCost: number,
  triesPerAdmit: number,
  solutions: string[]
) {
  const challengePda = pdaForChallenge(creator, id)
  const redeemPda = pdaForRedeem(challengePda)
  const hashedSolutions = doubleHashSolutions(solutions)

  const accounts = {
    payer,
    creator,
    challengePda,
    redeemPda,
  }
  let args: CreateChallengeInstructionArgs = {
    id,
    admitCost,
    triesPerAdmit,
    redeem: redeemPda,
    solutions: hashedSolutions,
  }
  return createCreateChallengeInstruction(accounts, args)
}

// -----------------
// Add Solutions
// -----------------
export function addSolutions(
  payer: PublicKey,
  creator: PublicKey,
  id: string,
  solutions: string[]
) {
  const challengePda = pdaForChallenge(creator, id)
  const hashedSolutions = doubleHashSolutions(solutions)

  const accounts = {
    payer,
    creator,
    challengePda,
  }
  let args: AddSolutionsInstructionArgs = {
    id,
    solutions: hashedSolutions,
  }
  return createAddSolutionsInstruction(accounts, args)
}

// -----------------
// Start Challenge
// -----------------
export function startChallenge(creator: PublicKey, id: string) {
  const challengePda = pdaForChallenge(creator, id)
  const accounts = {
    creator,
    challengePda,
  }
  return createStartChallengeInstruction(accounts, { id })
}

// -----------------
// Admit Challenger
// -----------------
export function admitChallenger(
  payer: PublicKey,
  creator: PublicKey,
  challengeId: string,
  challenger: PublicKey
) {
  const challengePda = pdaForChallenge(creator, challengeId)
  const challengerPda = pdaForChallenger(challengePda, challenger)

  const accounts: AdmitChallengerInstructionAccounts = {
    payer,
    creator,
    challengePda,
    challenger,
    challengerPda,
  }
  return createAdmitChallengerInstruction(accounts, { challengePda })
}

// -----------------
// Redeem
// -----------------
export async function redeem(
  payer: PublicKey,
  creator: PublicKey,
  challengeId: string,
  challenger: PublicKey,
  solution: string
) {
  const challengePda = pdaForChallenge(creator, challengeId)
  const challengerPda = pdaForChallenger(challengePda, challenger)
  const redeem = Redeem.forChallengeWith(creator, challengeId)

  const accounts: RedeemInstructionAccounts = {
    payer,
    challengePda,
    challenger,
    challengerPda,
    redeem: redeem.pda,
    redeemAta: await redeem.ata(challenger),
    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
  }
  const hashedSolution = hashSolution(solution)
  return createRedeemInstruction(accounts, { solution: hashedSolution })
}
