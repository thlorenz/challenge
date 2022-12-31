import { Amman } from '@metaplex-foundation/amman-client'
import { cusper } from '../../src/errors'

import { PROGRAM_ADDRESS } from '../../src/generated'

export const amman = Amman.instance({
  knownLabels: { [PROGRAM_ADDRESS]: 'Challenge Program' },
  errorResolver: cusper,
})
