import { pdaForChallenger } from 'src/common/pda'
import { HasPda } from 'src/framework/types'
import { Challenger as ChallengerAccount, ChallengerArgs } from '../generated'

export class Challenger implements HasPda {
  private _inner: ChallengerAccount
  constructor(args: ChallengerArgs) {
    this._inner = ChallengerAccount.fromArgs(args)
  }

  get pda() {
    return pdaForChallenger(this._inner.challengePda, this._inner.authority)
  }
}
