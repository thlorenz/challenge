import { LOCALHOST } from '@metaplex-foundation/amman-client'
import { ConfirmOptions, Connection, PublicKey } from '@solana/web3.js'
import test from 'tape'

export * from './amman'
export * from './asserts'
export * from './setup'

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
export function getLamports(address: PublicKey) {
  const connection = new Connection(LOCALHOST, 'confirmed')
  return connection.getAccountInfo(address).then((x) => x?.lamports)
}
