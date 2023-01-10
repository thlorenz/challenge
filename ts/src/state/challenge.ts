import {
  AccountInfo,
  Commitment,
  Connection,
  GetAccountInfoConfig,
  LAMPORTS_PER_SOL,
  PublicKey,
} from '@solana/web3.js'
import BN from 'bn.js'
import { pdaForChallenge, pdaForRedeem } from '../common/pda'
import { HasPda } from '../framework/types'
import {
  Challenge as ChallengeAccount,
  ChallengeArgs,
  Challenger as ChallengerAccount,
} from '../generated'
import { Challenger } from './challenger'

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
      .then((accounts) =>
        accounts.reduce((map, { pubkey, account }) => {
          map.set(pubkey.toBase58(), Challenge.fromAccountInfo(account))
          return map
        }, new Map<string, Challenge>())
      )
  }

  static async findByCreatorWithStats(
    connection: Connection,
    creator: PublicKey
  ) {
    const challenges = await Challenge.findByCreator(connection, creator)
    const map = new Map<string, ChallengeWithStats>()
    for (const [key, challenge] of challenges) {
      const challengers = new Map(
        (await challenge.findAdmittedChallengers(connection)).map(
          ({ pubkey, account }) => [
            pubkey.toBase58(),
            Challenger.fromAccountInfo(account),
          ]
        )
      )

      map.set(key, new ChallengeWithStats(challenge, challengers))
    }
    return map
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

  get pdaForRedeem() {
    return pdaForRedeem(this.pda)
  }

  get state() {
    return this._inner
  }

  get admitCost() {
    return this._inner.admitCost
  }
}

export class ChallengeWithStats {
  public readonly redeemers: Map<string, Challenger> = new Map()

  constructor(
    public readonly challenge: Challenge,
    public readonly challengers: Map<string, Challenger>
  ) {
    for (const [key, challenger] of challengers) {
      if (challenger.redeemed) {
        this.redeemers.set(key, challenger)
      }
    }
  }

  get admitted() {
    return this.challengers.size
  }

  get redeemed() {
    return this.redeemers.size
  }

  get feesPaid() {
    return new BN(this.admitted)
      .mul(new BN(this.challenge.admitCost))
      .div(new BN(LAMPORTS_PER_SOL))
      .toNumber()
  }

  pretty() {
    const challengers: Record<string, object> = {}
    for (const [key, challenger] of this.challengers) {
      challengers[key] = challenger.pretty()
    }
    return {
      ...this.challenge.pretty(),
      challengers,
      admitted: this.challengers.size,
      redeemed: this.redeemed,
      feesPaid: this.feesPaid,
    }
  }
}
