import { Keypair, PublicKey, Transaction } from '@solana/web3.js'
import { pdaForChallenge } from 'src/common/pda'
import { createChallenge } from 'src/ixs'
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

test('01_init-challenge', async (t) => {
  const {
    connection: _,
    creator,
    creatorPair: __,
    creatorTxHandler,
    challengeId,
    challenge: ___,
    challengePair: ____,
    challengePda: _____,
  } = await initChallengeAndCreator()

  const admitCost = 1
  const triesPerAdmit = 3

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
      .sendAndConfirmTransaction(tx, [], 'tx: init mint')
      .assertSuccess(t)
  }
})
