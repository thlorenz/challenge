import {
  AccountInfo,
  Commitment,
  Connection,
  GetAccountInfoConfig,
  PublicKey,
} from '@solana/web3.js'
import { pdaForChallenge } from '../common/pda'
import { HasPda } from '../framework/types'
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

  // Diagnositics

  static findAll(connection: Connection) {
    return ChallengeAccount.gpaBuilder().run(connection)
  }

  static findByCreator(connection: Connection, creator: PublicKey) {
    return ChallengeAccount.gpaBuilder()
      .addFilter('authority', creator)
      .run(connection)
  }

  findAdmittedChallengers(connection: Connection) {
    return ChallengerAccount.gpaBuilder()
      .addFilter('challengePda', this.pda)
      .run(connection)
  }

  static fromAccountInfo(
    accountInfo: AccountInfo<Buffer>,
    offset = 0
  ): Challenge {
    const [account] = ChallengeAccount.fromAccountInfo(accountInfo, offset)
    return new Challenge(account)
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
