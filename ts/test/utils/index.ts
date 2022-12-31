import { LOCALHOST } from '@metaplex-foundation/amman-client'
import { ConfirmOptions, Connection, Keypair, PublicKey } from '@solana/web3.js'
import test from 'tape'
import { amman } from './amman'

export * from './amman'
export * from './asserts'

export function killStuckProcess() {
  test.onFinish(() => process.exit(0))
}

export function inspect(obj: any) {
  console.log(JSON.stringify(obj, null, 2))
}

export const SKIP_PREFLIGHT: ConfirmOptions = {
  skipPreflight: true,
  commitment: 'singleGossip',
}

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
