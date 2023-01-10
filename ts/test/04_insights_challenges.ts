import { Challenge } from '../src/state/challenge'
import test, { Test } from 'tape'
import {
  killStuckProcess,
  createEmptyChallenge,
  startChallengeWithSolutions,
  setupAndAdmitChallenger,
} from './utils'
import { PublicKey, Transaction } from '@solana/web3.js'
import { PayerTransactionHandler } from '@metaplex-foundation/amman-client'
import { createChallenge, redeem } from '../src/ixs'
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

// TODO(thlorenz): Why is this failing on de-serialization?
test.skip('insights-challenge: retrieving all challenges', async (t) => {
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

test('insights-challenge: retrieving all challenges by creator with stats', async (t) => {
  t.comment('1. Create and start Challenge')

  const {
    connection,
    creator,
    challengeId,
    challengePda,
    admitCost,
    triesPerAdmit,
  } = await startChallengeWithSolutions(t)

  t.comment('2. Admit two challengers')
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

  // Checks after admitting two challengers
  {
    const challenges = await Challenge.findByCreatorWithStats(
      connection,
      creator
    )

    t.equal(challenges.size, 1, 'found one challenge for creator')
    const { challengers, ...challenge } = challenges
      .get(challengePda.toBase58())!
      .pretty()

    spok(t, challenge, {
      authority: creator.toBase58(),
      id: 'fst-challenge',
      started: true,
      finished: false,
      admitCost,
      triesPerAdmit,
      redeem: pdaForRedeem(challengePda).toBase58(),
      solving: 0,
      admitted: 2,
      redeemed: 0,
    })

    const challenger1 = challengers[cpda1.toBase58()]
    spok(t, challenger1, {
      authority: c1.toBase58(),
      challengePda: challengePda.toBase58(),
      triesRemaining: triesPerAdmit,
      redeemed: false,
    })
    const challenger2 = challengers[cpda2.toBase58()]
    spok(t, challenger2, {
      authority: c2.toBase58(),
      challengePda: challengePda.toBase58(),
      triesRemaining: triesPerAdmit,
      redeemed: false,
    })
  }

  t.comment('3. First Challenger solves the challenge')
  {
    let ix = await redeem(c1, creator, challengeId, c1, 'hello')

    const tx = new Transaction().add(ix)
    await ctx1
      .sendAndConfirmTransaction(tx, [], 'tx: challenger 1 solves challenge')
      .assertSuccess(t)

    const challenges = await Challenge.findByCreatorWithStats(
      connection,
      creator
    )

    t.equal(challenges.size, 1, 'found one challenge for creator')
    const { challengers, ...challenge } = challenges
      .get(challengePda.toBase58())!
      .pretty()

    // solving goes up
    spok(t, challenge, {
      authority: creator.toBase58(),
      id: 'fst-challenge',
      started: true,
      finished: false,
      solving: 1,
      admitted: 2,
      redeemed: 1,
    })

    const challenger1 = challengers[cpda1.toBase58()]
    spok(t, challenger1, {
      authority: c1.toBase58(),
      challengePda: challengePda.toBase58(),
      triesRemaining: triesPerAdmit - 1,
      redeemed: true,
    })
  }

  {
    t.comment('4. Second Challenger solves the challenge with wrong solution')
    let ix = await redeem(c2, creator, challengeId, c2, 'hello')

    const tx = new Transaction().add(ix)
    await ctx2
      .sendAndConfirmTransaction(tx, [], 'tx: challenger 2 solves challenge')
      .assertSuccess(t)

    const challenges = await Challenge.findByCreatorWithStats(
      connection,
      creator
    )

    t.equal(challenges.size, 1, 'found one challenge for creator')
    const { challengers, ...challenge } = challenges
      .get(challengePda.toBase58())!
      .pretty()

    spok(t, challenge, {
      authority: creator.toBase58(),
      id: 'fst-challenge',
      started: true,
      finished: false,
      solving: 1,
      admitted: 2,
      redeemed: 1,
    })

    // looses a try, but not redeemed
    const challenger2 = challengers[cpda2.toBase58()]
    spok(t, challenger2, {
      authority: c2.toBase58(),
      challengePda: challengePda.toBase58(),
      triesRemaining: triesPerAdmit - 1,
      redeemed: false,
    })
  }
})
