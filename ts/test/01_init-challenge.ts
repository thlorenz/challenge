import { Keypair, PublicKey, Transaction } from '@solana/web3.js'
import spok from 'spok'
import { pdaForChallenge } from 'src/common/pda'
import { addSolutions, createChallenge, startChallenge } from 'src/ixs'
import { Challenge } from 'src/state/challenge'
import { Redeem } from 'src/state/redeem'
import test from 'tape'
import { amman, killStuckProcess, setupCreator } from './utils'

killStuckProcess()

async function initChallengeAndCreator(challengeId = 'fst-challenge') {
  const { connection, creator, creatorPair, creatorTxHandler } =
    await setupCreator()

  const [challenge, challengePair]: [PublicKey, Keypair, string] =
    await amman.addr.genLabeledKeypair('challenge')

  const challengePda = pdaForChallenge(creator, challengeId)
  await amman.addr.addLabel('challengePda', challengePda)

  const redeem = Redeem.forChallengeWith(creator, challengeId)
  await amman.addr.addLabel('redeem mint', redeem.pda)

  return {
    connection,
    creator,
    creatorPair,
    creatorTxHandler,
    challengeId,
    challenge,
    challengePair,
    challengePda,
  }
}

test('init-challenge: adding solutions separately, then starting', async (t) => {
  const {
    connection,
    creator,
    creatorPair: __,
    creatorTxHandler,
    challengeId,
    challenge: ___,
    challengePair: ____,
    challengePda,
  } = await initChallengeAndCreator()

  const admitCost = 1
  const triesPerAdmit = 3

  async function challenge() {
    const x = await Challenge.fromAccountAddress(connection, challengePda)
    return x.pretty()
  }

  // 1. Create challenge
  {
    const ix = createChallenge(
      creator,
      creator,
      challengeId,
      admitCost,
      triesPerAdmit,
      []
    )
    const tx = new Transaction().add(ix)

    await creatorTxHandler
      .sendAndConfirmTransaction(tx, [], 'tx: create challenge')
      .assertSuccess(t)

    spok(t, await challenge(), {
      authority: creator.toBase58(),
      id: 'fst-challenge',
      started: false,
      finished: false,
      admitCost: 1,
      triesPerAdmit: 3,
    })
  }

  // 2. Add solutions
  {
    const solutions = ['hello', 'world']
    const ix = addSolutions(creator, creator, challengeId, solutions)

    const tx = new Transaction().add(ix)

    await creatorTxHandler
      .sendAndConfirmTransaction(tx, [], 'tx: add solutions')
      .assertSuccess(t)

    t.equal((await challenge()).solutions.length, 2, 'added 2 solutions')
  }

  // 3. Start challenge
  {
    const ix = startChallenge(creator, challengeId)
    const tx = new Transaction().add(ix)

    await creatorTxHandler
      .sendAndConfirmTransaction(tx, [], 'tx: start challenge')
      .assertSuccess(t)

    spok(t, await challenge(), {
      authority: creator.toBase58(),
      id: 'fst-challenge',
      started: true,
      finished: false,
      admitCost: 1,
      triesPerAdmit: 3,
    })
    t.equal((await challenge()).solutions.length, 2, 'still has 2 solutions')
  }
})

test('init-challenge: adding solutions on creation, then starting', async (t) => {
  const {
    connection,
    creator,
    creatorPair: __,
    creatorTxHandler,
    challengeId,
    challenge: ___,
    challengePair: ____,
    challengePda,
  } = await initChallengeAndCreator()

  const admitCost = 1
  const triesPerAdmit = 3

  async function challenge() {
    const x = await Challenge.fromAccountAddress(connection, challengePda)
    return x.pretty()
  }

  // 1. Create challenge
  {
    const solutions = ['hello', 'world']
    const ix = createChallenge(
      creator,
      creator,
      challengeId,
      admitCost,
      triesPerAdmit,
      solutions
    )
    const tx = new Transaction().add(ix)

    await creatorTxHandler
      .sendAndConfirmTransaction(tx, [], 'tx: create challenge')
      .assertSuccess(t)

    spok(t, await challenge(), {
      authority: creator.toBase58(),
      id: 'fst-challenge',
      started: false,
      finished: false,
      admitCost: 1,
      triesPerAdmit: 3,
    })
    t.equal((await challenge()).solutions.length, 2, 'added 2 solutions')
  }

  // 2. Start challenge
  {
    const ix = startChallenge(creator, challengeId)
    const tx = new Transaction().add(ix)

    await creatorTxHandler
      .sendAndConfirmTransaction(tx, [], 'tx: start challenge')
      .assertSuccess(t)

    spok(t, await challenge(), {
      authority: creator.toBase58(),
      id: 'fst-challenge',
      started: true,
      finished: false,
      admitCost: 1,
      triesPerAdmit: 3,
    })
    t.equal((await challenge()).solutions.length, 2, 'still has 2 solutions')
  }
})
