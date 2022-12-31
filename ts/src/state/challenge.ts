import {
  Commitment,
  Connection,
  GetAccountInfoConfig,
  PublicKey,
} from '@solana/web3.js'
import { pdaForChallenge } from 'src/common/pda'
import { HasPda } from 'src/framework/types'
import {
  Challenge as ChallengeAccount,
  ChallengeArgs,
  Challenger as ChallengerAccount,
} from '../generated'

export class Challenge implements HasPda {
  private readonly _inner: ChallengeAccount

  constructor(args: ChallengeArgs) {
    this._inner = ChallengeAccount.fromArgs(args)
  }

  admittedChallengers(connection: Connection) {
    return ChallengerAccount.gpaBuilder()
      .addFilter('challengePda', this.pda)
      .run(connection)
  }

  static async fromAccountAddress(
    connection: Connection,
    address: PublicKey,
    commitmentOrConfig?: Commitment | GetAccountInfoConfig
  ): Promise<Challenge> {
    const account = await ChallengeAccount.fromAccountAddress(
      connection,
      address,
      commitmentOrConfig
    )
    return new Challenge(account)
  }

  pretty() {
    return this._inner.pretty()
  }

  get pda() {
    return pdaForChallenge(this._inner.authority, this._inner.id)
  }
}
