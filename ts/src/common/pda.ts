import { PublicKey } from '@solana/web3.js'
import { PROGRAM_ID } from '../generated'

// Eventually shank will add info from derived from the below to the IDL which solita will use to generate PDA and seed
// methods.
// #[seeds(
//     "challenge",
//     creator("The authority managing the challenge, usually the creator"),
//     challenge_id(
//         "Unique id of the challenge. The same creator cannot reuse the same id for different challenges.",
//         str
//     )
// )]
export function pdaForChallenge(
  creator: PublicKey,
  challengeId: string
): PublicKey {
  const [pda] = PublicKey.findProgramAddressSync(
    [Buffer.from('challenge'), creator.toBuffer(), Buffer.from(challengeId)],
    PROGRAM_ID
  )
  return pda
}

// #[seeds(
//     "challenge",
//     challenge_pda("The challenge PDA that the challenger wants to solve."),
//     challenger("The address attempting to solve the challenge")
// )]
export function pdaForChallenger(
  challengePda: PublicKey,
  challenger: PublicKey
): PublicKey {
  const [pda] = PublicKey.findProgramAddressSync(
    [Buffer.from('challenge'), challengePda.toBuffer(), challenger.toBuffer()],
    PROGRAM_ID
  )
  return pda
}

// #[seeds("challenge", challenge_pda("The PDA of the challenge"))]
export function pdaForRedeem(challengePda: PublicKey): PublicKey {
  const [pda] = PublicKey.findProgramAddressSync(
    [Buffer.from('challenge'), challengePda.toBuffer()],
    PROGRAM_ID
  )
  return pda
}
