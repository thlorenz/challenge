import { Challenge, ChallengeWithStats } from '../src/state/challenge'
import test, { Test } from 'tape'
import {
  killStuckProcess,
  createEmptyChallenge,
  startChallengeWithSolutions,
  setupChallenger,
  setupAndAdmitChallenger,
} from './utils'
import { PublicKey, Transaction } from '@solana/web3.js'
import { PayerTransactionHandler } from '@metaplex-foundation/amman-client'
import { createChallenge } from '../src/ixs'
import { pdaForRedeem } from '../src/common/pda'
import spok from 'spok'

killStuckProcess()

function createChallengeTx(
  t: Test,
  txHandler: PayerTransactionHandler,
  args: {
    creator: PublicKey
    id: string
    admitCost: number
    triesPerAdmit: number
  }
) {
  const { creator, id, admitCost, triesPerAdmit } = args
  const createIx = createChallenge(
    creator,
    creator,
    id,
    admitCost,
    triesPerAdmit,
    []
  )

  const tx = new Transaction().add(createIx)

  return txHandler
    .sendAndConfirmTransaction(tx, [], 'tx: re-create challenge ' + id)
    .assertSuccess(t)
}

test('insights-challenge: retrieving all challenges', async (t) => {
  const { creator: c1, connection } = await createEmptyChallenge(t)
  const { creator: c2 } = await createEmptyChallenge(t)
  const { creator: c3 } = await createEmptyChallenge(t)

  const challenges = (await Challenge.findAll(connection)).map(({ account }) =>
    Challenge.fromAccountInfo(account).pretty()
  )
  t.ok(challenges.length >= 3, 'finds at least three challenges')
  t.ok(
    challenges.some((c) => c.authority === c1.toBase58()),
    'included challenge of first creator'
  )
  t.ok(
    challenges.some((c) => c.authority === c2.toBase58()),
    'included challenge of second creator'
  )
  t.ok(
    challenges.some((c) => c.authority === c3.toBase58()),
    'included challenge of third creator'
  )
})

test('insights-challenge: retrieving all challenges by creator', async (t) => {
  const {
    connection,
    creatorTxHandler,
    creator: c1,
    admitCost,
    triesPerAdmit,
    challengeId,
  } = await createEmptyChallenge(t)
  const args = {
    creator: c1,
    id: `${challengeId}-snd`,
    admitCost,
    triesPerAdmit,
  }
  await createChallengeTx(t, creatorTxHandler, args)

  const { creator: c2 } = await createEmptyChallenge(t)

  t.equal(
    (await Challenge.findByCreator(connection, c1)).size,
    2,
    'found two challenges for first creator'
  )
  t.equal(
    (await Challenge.findByCreator(connection, c2)).size,
    1,
    'found one challenge for second creator'
  )
})

test.only('insights-challenge: retrieving all challenges by creator with stats', async (t) => {
  const {
    connection,
    creator,
    challengeId,
    challengePda,
    admitCost,
    triesPerAdmit,
  } = await startChallengeWithSolutions(t)

  // 1. Admit first challenger
  {
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

    const challenges = await Challenge.findByCreatorWithStats(
      connection,
      creator
    )

    t.equal(challenges.size, 1, 'found one challenge for creator')
    const { challenge, challengers } = challenges.get(challengePda.toBase58())!

    spok(t, challenge.pretty(), {
      authority: creator.toBase58(),
      id: 'fst-challenge',
      started: true,
      finished: false,
      admitCost,
      triesPerAdmit,
      redeem: pdaForRedeem(challengePda).toBase58(),
      solving: 0,
    })

    const challenger1 = challengers.get(cpda1.toBase58())!
    spok(t, challenger1.pretty(), {
      authority: c1.toBase58(),
      challengePda: challengePda.toBase58(),
      triesRemaining: triesPerAdmit,
      redeemed: false,
    })
    const challenger2 = challengers.get(cpda2.toBase58())!
    spok(t, challenger2.pretty(), {
      authority: c2.toBase58(),
      challengePda: challengePda.toBase58(),
      triesRemaining: triesPerAdmit,
      redeemed: false,
    })
    // TODO(thlorenz): aredeem challengers
  }
})
