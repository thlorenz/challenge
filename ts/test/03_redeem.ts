import { PublicKey, Transaction } from '@solana/web3.js'
import { redeem } from '../src/ixs'
import test from 'tape'
import {
  killStuckProcess,
  startChallengeWithSolutions,
  setupAndAdmitChallenger,
} from './utils'
import { Challenger } from '../src/state/challenger'
import { Redeem } from '../src/state/redeem'
import spok from 'spok'

killStuckProcess()

test('redeem: two challengers each redeeming solutions', async (t) => {
  const { connection, creator, challengeId, challengePda } =
    await startChallengeWithSolutions(t)
  const redeemMint = Redeem.forChallengeWith(creator, challengeId).pda

  async function challengerInfo(challenger: PublicKey, cpda: PublicKey) {
    const acc = await Challenger.fromAccountAddress(connection, cpda)

    const toks = await connection
      .getTokenAccountsByOwner(challenger, {
        mint: redeemMint,
      })
      .then((x) => x.value?.length ?? 0)
    return { acc: acc.pretty(), toks }
  }

  // 1. Admit challengers
  const {
    challenger: c1,
    challengerPda: cpda1,
    challengerTxHandler: ctx1,
  } = await setupAndAdmitChallenger(t, creator, challengeId)

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

    spok(t, await challengerInfo(c1, cpda1), {
      acc: {
        authority: c1.toBase58(),
        challengePda: challengePda.toBase58(),
        triesRemaining: 2,
        redeemed: false,
      },
      toks: 0,
    })
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

    spok(t, await challengerInfo(c1, cpda1), {
      acc: {
        authority: c1.toBase58(),
        challengePda: challengePda.toBase58(),
        triesRemaining: 1,
        redeemed: true,
      },
      toks: 1,
    })
  }

  // 4. Second Challenger attempts same solution again
  {
    const ix = await redeem(c2, creator, challengeId, c2, 'hello')
    const tx = new Transaction().add(ix)

    await ctx2
      .sendAndConfirmTransaction(
        tx,
        [],
        'tx: second challenger redeems with same now outdated solution'
      )
      .assertSuccess(t)

    spok(t, await challengerInfo(c2, cpda2), {
      acc: {
        authority: c2.toBase58(),
        challengePda: challengePda.toBase58(),
        triesRemaining: 2,
        redeemed: false,
      },
      toks: 0,
    })
  }

  // 5. Second Challenger attempts second solution
  {
    const ix = await redeem(c2, creator, challengeId, c2, 'world')
    const tx = new Transaction().add(ix)

    await ctx2
      .sendAndConfirmTransaction(
        tx,
        [],
        'tx: second challenger redeems with second solution'
      )
      .assertSuccess(t)

    spok(t, await challengerInfo(c2, cpda2), {
      acc: {
        authority: c2.toBase58(),
        challengePda: challengePda.toBase58(),
        triesRemaining: 1,
        redeemed: true,
      },
      toks: 1,
    })
  }
})

// TODO(thlorenz): add tests for running out of tries, etc. unless already covered via banks client tests
