import { Challenge } from '../src/state/challenge'
import test, { Test } from 'tape'
import { killStuckProcess, createEmptyChallenge } from './utils'
import { PublicKey, Transaction } from '@solana/web3.js'
import { PayerTransactionHandler } from '@metaplex-foundation/amman-client'
import { createChallenge } from '../src/ixs'

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
    (await Challenge.findByCreator(connection, c1)).length,
    2,
    'found two challenges for first creator'
  )
  t.equal(
    (await Challenge.findByCreator(connection, c2)).length,
    1,
    'found one challenge for second creator'
  )
})
