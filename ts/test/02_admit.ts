import { Transaction } from '@solana/web3.js'
import spok from 'spok'
import { admitChallenger } from '../src/ixs'
import test from 'tape'
import {
  killStuckProcess,
  startChallengeWithSolutions,
  setupChallenger,
} from './utils'
import { Challenge } from '../src/state/challenge'
import { Challenger } from '../src/state/challenger'
import { PROGRAM_ADDRESS } from '../src/generated'

killStuckProcess()

test('admit: two challengers', async (t) => {
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
    const { challenger, challengerPda, challengerTxHandler } =
      await setupChallenger(challengePda)

    const creatorLamps = await connection.getBalance(creator)

    const ix = admitChallenger(challenger, creator, challengeId, challenger)
    const tx = new Transaction().add(ix)

    await challengerTxHandler
      .sendAndConfirmTransaction(tx, [], 'tx: admit first challenger')
      .assertSuccess(t)

    t.equal(
      await connection.getBalance(creator),
      creatorLamps + admitCost,
      'creator earns admit cost'
    )
    const challenge = await Challenge.fromAccountAddress(
      connection,
      challengePda
    )
    const admitteds = await challenge.admittedChallengers(connection)
    t.equal(admitteds.length, 1, 'one admitted challenger')

    const admitted = admitteds[0]
    t.equal(
      admitted.pubkey.toBase58(),
      challengerPda.toBase58(),
      'admitted challenger pda'
    )
    t.equal(
      admitted.account.owner.toBase58(),
      PROGRAM_ADDRESS,
      'challenger owned by program'
    )

    const challengerData = await Challenger.fromAccountAddress(
      connection,
      challengerPda
    )
    spok(t, challengerData.pretty(), {
      $topic: 'challenger data',
      triesRemaining: triesPerAdmit,
      authority: challenger.toBase58(),
      challengePda: challengePda.toBase58(),
      redeemed: false,
    })
  }

  // 2. Admit second challenger
  {
    const { challenger, challengerTxHandler } = await setupChallenger(
      challengePda
    )

    const creatorLamps = await connection.getBalance(creator)

    const ix = admitChallenger(challenger, creator, challengeId, challenger)
    const tx = new Transaction().add(ix)

    await challengerTxHandler
      .sendAndConfirmTransaction(tx, [], 'tx: admit second challenger')
      .assertSuccess(t)

    t.equal(
      await connection.getBalance(creator),
      creatorLamps + admitCost,
      'creator earns admit cost'
    )
    const challenge = await Challenge.fromAccountAddress(
      connection,
      challengePda
    )
    const admitteds = await challenge.admittedChallengers(connection)
    t.equal(admitteds.length, 2, 'two admitted challengers')
  }
})
