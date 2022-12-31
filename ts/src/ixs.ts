import { PublicKey } from '@solana/web3.js'
import { pdaForChallenge, pdaForRedeem } from './common/pda'
import { hashSolutions } from './common/solution'
import {
  CreateChallengeInstructionArgs,
  createCreateChallengeInstruction,
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
    // @ts-ignore
    solutions: hashedSolutions,
  }
  return createCreateChallengeInstruction(accounts, args)
}
