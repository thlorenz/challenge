import { createHash } from 'crypto'

function hash(input: string): Buffer {
  const sha256 = createHash('sha256')
  sha256.update(input)
  sha256.end()
  return sha256.read()
}

function hashSolutionToU8Array(solution: Buffer): Uint8Array {
  return Uint8Array.from(hash(solution.toString('hex')))
}

export function hashSolution(solution: string): number[] {
  return Array.from(hashSolutionToU8Array(Buffer.from(solution)))
}

export function hashSolutions(solutions: string[]): number[][] {
  return solutions.map((s) => {
    const challengerSends = hash(s)
    // program stores
    const uintArray = hashSolutionToU8Array(challengerSends)
    return Array.from(uintArray)
  })
}
