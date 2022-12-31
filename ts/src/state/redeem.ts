import { getAssociatedTokenAddress } from '@solana/spl-token'
import { PublicKey } from '@solana/web3.js'
import { pdaForChallenge, pdaForRedeem } from 'src/common/pda'
import { HasPda } from 'src/framework/types'
import { Redeem as RedeemAccount, RedeemArgs } from '../generated'

export class Redeem implements HasPda {
  private _inner: RedeemAccount

  constructor(args: RedeemArgs) {
    this._inner = RedeemAccount.fromArgs(args)
  }

  static forChallengeWith(creator: PublicKey, challengeId: string) {
    const challengePDA = pdaForChallenge(creator, challengeId)
    const args: RedeemArgs = {
      challengePda: challengePDA,
      pda: pdaForRedeem(challengePDA),
    }
    return new Redeem(args)
  }

  get pda() {
    return pdaForRedeem(this._inner.challengePda)
  }

  ata(recvr: PublicKey): Promise<PublicKey> {
    return getAssociatedTokenAddress(recvr, this._inner.pda)
  }
}
