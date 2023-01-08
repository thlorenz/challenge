import { LOCALHOST } from '@metaplex-foundation/amman-client'
import {
  Connection,
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  Transaction,
} from '@solana/web3.js'
import { pdaForChallenge, pdaForChallenger } from '../../src/common/pda'
import { admitChallenger, createChallenge, startChallenge } from '../../src/ixs'
import { Redeem } from '../../src/state/redeem'
import { Test } from 'tape'
import { amman } from './amman'

export async function setupCreator() {
  const connection = new Connection(LOCALHOST, 'confirmed')

  const [creator, creatorPair]: [PublicKey, Keypair, string] =
    await amman.addr.genLabeledKeypair('creator')
  await amman.airdrop(connection, creator, 5)

  const creatorTxHandler = amman.payerTransactionHandler(
    connection,
    creatorPair
  )

  return { connection, creator, creatorPair, creatorTxHandler }
}

export async function setupChallenger(challengePda: PublicKey) {
  const connection = new Connection(LOCALHOST, 'confirmed')

  const [challenger, challengerPair]: [PublicKey, Keypair, string] =
    await amman.addr.genLabeledKeypair('challenger')
  await amman.airdrop(connection, challenger, 5)

  const challengerPda = pdaForChallenger(challengePda, challenger)
  await amman.addr.addLabel('challengerPda', challengerPda)

  const challengerTxHandler = amman.payerTransactionHandler(
    connection,
    challengerPair
  )

  return {
    connection,
    challenger,
    challengerPair,
    challengerPda,
    challengerTxHandler,
  }
}

export async function initChallengeAndCreator(challengeId = 'fst-challenge') {
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
export async function createEmptyChallenge(
  t: Test,
  opts = {
    challengeId: 'challenge',
    admitCost: 0.25 * LAMPORTS_PER_SOL, // 0.25 SOL ~ $2.50
    triesPerAdmit: 3,
  }
) {
  const { challengeId, admitCost, triesPerAdmit } = opts
  const {
    connection,
    creator,
    creatorPair,
    creatorTxHandler,
    challenge,
    challengePair,
    challengePda,
  } = await initChallengeAndCreator(challengeId)

  const createIx = createChallenge(
    creator,
    creator,
    challengeId,
    admitCost,
    triesPerAdmit,
    []
  )

  const tx = new Transaction().add(createIx)

  await creatorTxHandler
    .sendAndConfirmTransaction(tx, [], 'tx: create challenge ' + challengeId)
    .assertSuccess(t)

  return {
    connection,
    creator,
    creatorPair,
    creatorTxHandler,
    challengeId,
    challenge,
    challengePair,
    challengePda,
    admitCost,
    triesPerAdmit,
  }
}

export async function startChallengeWithSolutions(
  t: Test,
  opts = {
    challengeId: 'fst-challenge',
    admitCost: 0.25 * LAMPORTS_PER_SOL, // 0.25 SOL ~ $2.50
    triesPerAdmit: 3,
  }
) {
  const { challengeId, admitCost, triesPerAdmit } = opts

  const {
    connection,
    creator,
    creatorPair,
    creatorTxHandler,
    challenge,
    challengePair,
    challengePda,
  } = await initChallengeAndCreator(challengeId)

  const solutions = ['hello', 'world']
  const createIx = createChallenge(
    creator,
    creator,
    challengeId,
    admitCost,
    triesPerAdmit,
    solutions
  )

  const startIx = startChallenge(creator, challengeId)
  const tx = new Transaction().add(createIx).add(startIx)

  await creatorTxHandler
    .sendAndConfirmTransaction(tx, [], 'tx: create and start challenge')
    .assertSuccess(t)

  return {
    connection,
    creator,
    creatorPair,
    creatorTxHandler,
    challengeId,
    challenge,
    challengePair,
    challengePda,
    solutions,
    admitCost,
    triesPerAdmit,
  }
}

export async function setupAndAdmitChallenger(
  t: Test,
  creator: PublicKey,
  challengeId: string
) {
  const challengePda = pdaForChallenge(creator, challengeId)
  const { challenger, challengerPair, challengerPda, challengerTxHandler } =
    await setupChallenger(challengePda)

  const redeem = Redeem.forChallengeWith(creator, challengeId)
  await amman.addr.addLabel('challengerRedeemATA', await redeem.ata(challenger))

  let ix = admitChallenger(challenger, creator, challengeId, challenger)
  const tx = new Transaction().add(ix)
  await challengerTxHandler
    .sendAndConfirmTransaction(tx, [], 'tx: admit challenger')
    .assertSuccess(t)

  return {
    challenger,
    challengerPair,
    challengerPda,
    challengerTxHandler,
  }
}
