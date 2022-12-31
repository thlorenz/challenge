import { pdaForChallenge } from 'src/common/pda'
import { HasPda } from 'src/framework/types'
import { Challenge as ChallengeAccount, ChallengeArgs } from '../generated'

export class Challenge implements HasPda {
  private _inner: ChallengeAccount
  constructor(args: ChallengeArgs) {
    this._inner = ChallengeAccount.fromArgs(args)
  }

  get pda() {
    return pdaForChallenge(this._inner.authority, this._inner.id)
  }
}
