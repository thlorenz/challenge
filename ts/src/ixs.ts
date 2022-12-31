import { PublicKey } from '@solana/web3.js'
import { pdaForChallenge, pdaForRedeem } from './common/pda'
import { hashSolutions } from './common/solution'
import {
  AddSolutionsInstructionArgs,
  createAddSolutionsInstruction,
  CreateChallengeInstructionArgs,
  createCreateChallengeInstruction,
  createStartChallengeInstruction,
} from './generated'

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
  const hashedSolutions = hashSolutions(solutions)

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

export function addSolutions(
  payer: PublicKey,
  creator: PublicKey,
  id: string,
  solutions: string[]
) {
  const challengePda = pdaForChallenge(creator, id)
  const hashedSolutions = hashSolutions(solutions)

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

export function startChallenge(creator: PublicKey, id: string) {
  const challengePda = pdaForChallenge(creator, id)
  const accounts = {
    creator,
    challengePda,
  }
  return createStartChallengeInstruction(accounts, { id })
}
