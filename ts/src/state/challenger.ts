import {
  AccountInfo,
  Commitment,
  Connection,
  GetAccountInfoConfig,
  PublicKey,
} from '@solana/web3.js'
import { pdaForChallenger } from '../common/pda'
import { HasPda } from '../framework/types'
import { Challenger as ChallengerAccount, ChallengerArgs } from '../generated'

export class Challenger implements HasPda {
  private _inner: ChallengerAccount
  constructor(args: ChallengerArgs) {
    this._inner = ChallengerAccount.fromArgs(args)
  }

  static fromAccountInfo(
    accountInfo: AccountInfo<Buffer>,
    offset = 0
  ): Challenger {
    const [account] = ChallengerAccount.fromAccountInfo(accountInfo, offset)
    return new Challenger(account)
  }

  static async fromAccountAddress(
    connection: Connection,
    address: PublicKey,
    commitmentOrConfig?: Commitment | GetAccountInfoConfig
  ): Promise<Challenger> {
    const account = await ChallengerAccount.fromAccountAddress(
      connection,
      address,
      commitmentOrConfig
    )
    return new Challenger(account)
  }

  pretty() {
    return this._inner.pretty()
  }

  get state() {
    return this._inner
  }

  get redeemed() {
    return this._inner.redeemed
  }

  get pda() {
    return pdaForChallenger(this._inner.challengePda, this._inner.authority)
  }

  static get getMinimumBalanceForRentExemption() {
    return ChallengerAccount.getMinimumBalanceForRentExemption
  }

  static get size() {
    return ChallengerAccount.byteSize
  }
}
