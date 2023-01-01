import { createHash } from 'crypto'

function hash(input: string | Buffer): Buffer {
  const sha256 = createHash('sha256')
  sha256.update(input)
  sha256.end()
  return sha256.read()
}

export function prettySolution(solution: number[]): string {
  const hexs = solution.map((x) => x.toString(16)).join(', ')
  return `[${hexs}]`
}

export function hashSolution(solution: string): number[] {
  const hashed = hash(Buffer.from(solution))
  return Array.from(hashed)
}

export function doubleHashSolution(solution: string): number[] {
  const hashed = hash(hash(solution))
  return Array.from(Uint8Array.from(hashed))
}

export function doubleHashSolutions(solutions: string[]): number[][] {
  return solutions.map(doubleHashSolution)
}
