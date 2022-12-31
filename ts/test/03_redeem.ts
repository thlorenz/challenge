import { Transaction } from '@solana/web3.js'
import { redeem } from '../src/ixs'
import test from 'tape'
import {
  killStuckProcess,
  startChallengeWithSolutions,
  setupAndAdmitChallenger,
} from './utils'

killStuckProcess()

test('redeem: two challengers each redeeming solutions', async (t) => {
  const { creator, challengeId } = await startChallengeWithSolutions(t)

  // 1. Admit challengers
  const { challenger: c1, challengerTxHandler: ctx1 } =
    await setupAndAdmitChallenger(t, creator, challengeId)

  const {
    challenger: c2,
    challengerPda: cpda2,
    challengerTxHandler: ctx2,
  } = await setupAndAdmitChallenger(t, creator, challengeId)

  // 2. First Challenger attempts incorrect solution
  {
    const ix = await redeem(c1, creator, challengeId, c1, 'wrong')
    const tx = new Transaction().add(ix)

    await ctx1
      .sendAndConfirmTransaction(
        tx,
        [],
        'tx: first challenger redeems with wrong solution'
      )
      .assertSuccess(t)
  }
  // 3. First Challenger attempts correct solution
  {
    const ix = await redeem(c1, creator, challengeId, c1, 'hello')
    const tx = new Transaction().add(ix)

    await ctx1
      .sendAndConfirmTransaction(
        tx,
        [],
        'tx: first challenger redeems with correct solution'
      )
      .assertSuccess(t)
  }
})
