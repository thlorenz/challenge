import { createHash } from 'crypto'

const sha256 = createHash('sha256')

function hash(input: string): string {
  return sha256.update(input).digest('hex')
}

export function hashSolutionChallengerSeeds(solution: string): Uint8Array {
  return Uint8Array.from(Buffer.from(hash(solution)))
}

export function hashSolutions(solutions: string[]): Uint8Array[] {
  return solutions.map((s) => {
    const challengerSends = hash(s)
    // program stores
    return hashSolutionChallengerSeeds(challengerSends)
  })
}
